//! Sends one request and reads its reply, behind a trait so tests can use a fake keyboard.

use std::thread;
use std::time::Duration;

use hidapi::HidDevice;
use serde::Serialize;

use super::error::KeyboardError;
use super::protocol::{self, Command, REPORT_LEN};

/// Pause before each send and each read; the firmware needs a moment between steps.
const SETTLE: Duration = Duration::from_millis(10);

/// A request is sent up to this many times if the reply doesn't echo it.
const ATTEMPTS: u32 = 5;

/// First wait before a resend; it doubles each time (25, 50, 100, 200 ms) so another app talking
/// to the keyboard, like MagCrate, has time to finish.
const RETRY_BASE: Duration = Duration::from_millis(25);

/// Raw access to the keyboard's feature reports. Buffers include the leading report ID byte.
pub trait Transport {
    fn send_feature(&mut self, report: &[u8]) -> Result<(), KeyboardError>;
    fn get_feature(&mut self, buf: &mut [u8]) -> Result<usize, KeyboardError>;
}

pub struct HidTransport {
    device: HidDevice,
}

impl HidTransport {
    pub fn new(device: HidDevice) -> Self {
        Self { device }
    }
}

impl Transport for HidTransport {
    fn send_feature(&mut self, report: &[u8]) -> Result<(), KeyboardError> {
        // Last line of defense: nothing on the never-send list reaches the USB driver.
        protocol::guard_outgoing(report)?;
        Ok(self.device.send_feature_report(report)?)
    }

    fn get_feature(&mut self, buf: &mut [u8]) -> Result<usize, KeyboardError> {
        Ok(self.device.get_feature_report(buf)?)
    }
}

/// One request and its reply, kept so the diagnostics screen can show the raw bytes.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub command: Command,
    pub name: &'static str,
    pub request: Vec<u8>,
    pub reply: Vec<u8>,
}

/// Sends a command and returns the reply, retrying when the reply belongs to something else.
pub fn exchange(transport: &mut dyn Transport, command: Command) -> Result<Exchange, KeyboardError> {
    let request = protocol::build_request(command)?;
    let mut last_error = None;
    for attempt in 0..ATTEMPTS {
        if attempt > 0 {
            thread::sleep(RETRY_BASE * 2u32.pow(attempt - 1));
        }
        match try_exchange(transport, command, &request) {
            Ok(reply) => {
                return Ok(Exchange {
                    command,
                    name: command.name(),
                    request: request.to_vec(),
                    reply,
                });
            }
            Err(err @ KeyboardError::BadReply { .. }) => {
                log::debug!("{err} (attempt {}), retrying", attempt + 1);
                last_error = Some(err);
            }
            Err(err) => return Err(err),
        }
    }
    Err(last_error.unwrap_or(KeyboardError::NotConnected))
}

fn try_exchange(
    transport: &mut dyn Transport,
    command: Command,
    request: &[u8; REPORT_LEN],
) -> Result<Vec<u8>, KeyboardError> {
    // Report ID 0 goes in front of the 64 data bytes.
    let mut outgoing = [0u8; REPORT_LEN + 1];
    outgoing[1..].copy_from_slice(request);
    protocol::guard_outgoing(&outgoing)?;
    thread::sleep(SETTLE);
    transport.send_feature(&outgoing)?;

    let mut incoming = [0u8; REPORT_LEN + 1];
    thread::sleep(SETTLE);
    let len = transport.get_feature(&mut incoming)?.min(incoming.len());
    let reply = incoming.get(1..len).unwrap_or_default().to_vec();
    protocol::check_echo(command, &reply)?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    /// Fake keyboard that records what it was sent and plays back queued replies.
    struct FakeKeyboard {
        sent: Vec<Vec<u8>>,
        replies: VecDeque<Vec<u8>>,
    }

    impl FakeKeyboard {
        fn new(replies: &[&[u8]]) -> Self {
            Self {
                sent: Vec::new(),
                replies: replies.iter().map(|r| r.to_vec()).collect(),
            }
        }
    }

    impl Transport for FakeKeyboard {
        fn send_feature(&mut self, report: &[u8]) -> Result<(), KeyboardError> {
            self.sent.push(report.to_vec());
            Ok(())
        }

        fn get_feature(&mut self, buf: &mut [u8]) -> Result<usize, KeyboardError> {
            let reply = self.replies.pop_front().ok_or(KeyboardError::NotConnected)?;
            buf[0] = 0;
            buf[1..=reply.len()].copy_from_slice(&reply);
            Ok(REPORT_LEN + 1)
        }
    }

    #[test]
    fn sends_report_id_zero_then_the_request() {
        let mut keyboard = FakeKeyboard::new(&[&[0x84, 0x01]]);
        let result = exchange(&mut keyboard, Command::GetProfile).unwrap();
        let sent = &keyboard.sent[0];
        assert_eq!(sent.len(), REPORT_LEN + 1);
        assert_eq!(&sent[..9], &[0x00, 0x84, 0, 0, 0, 0, 0, 0, 0x7B]);
        assert_eq!(result.reply[..2], [0x84, 0x01]);
        assert_eq!(result.reply.len(), REPORT_LEN);
    }

    #[test]
    fn retries_when_a_stale_reply_arrives() {
        let mut keyboard = FakeKeyboard::new(&[&[0x87], &[0x84, 0x02]]);
        let result = exchange(&mut keyboard, Command::GetProfile).unwrap();
        assert_eq!(result.reply[1], 0x02);
        assert_eq!(keyboard.sent.len(), 2);
    }

    #[test]
    fn recovers_when_another_app_answers_first_several_times() {
        // Like MagCrate reading at the same moment: four stale replies, then the right one.
        let mut keyboard = FakeKeyboard::new(&[&[0x84], &[0x87], &[0x84], &[0x8F], &[0x83, 0, 2]]);
        let result = exchange(&mut keyboard, Command::GetPollingRate).unwrap();
        assert_eq!(result.reply[2], 2);
        assert_eq!(keyboard.sent.len(), 5);
    }

    #[test]
    fn gives_up_after_five_wrong_replies() {
        let mut keyboard = FakeKeyboard::new(&[&[0x00], &[0x00], &[0x00], &[0x00], &[0x00], &[0x84]]);
        assert!(matches!(
            exchange(&mut keyboard, Command::GetProfile),
            Err(KeyboardError::BadReply { .. })
        ));
        assert_eq!(keyboard.sent.len(), 5);
    }

    #[test]
    fn usb_errors_are_not_retried() {
        let mut keyboard = FakeKeyboard::new(&[]);
        assert!(matches!(
            exchange(&mut keyboard, Command::GetProfile),
            Err(KeyboardError::NotConnected)
        ));
        assert_eq!(keyboard.sent.len(), 1);
    }
}
