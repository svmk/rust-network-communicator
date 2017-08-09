use mio::{Event,Token,Poll};
use mio;
use std::boxed::Box;
use curl::multi::Multi;
use std::time::Duration;
use curl::easy::Easy2;
use curl::multi::{Socket,SocketEvents,Easy2Handle};
use curl::multi::Events as CurlEvents;
use std::collections::HashMap;
use std::sync::mpsc::SyncSender;
use task::is_terminate_task;
use super::Task;
use super::Error as TaskError;
mod error;
mod config;
pub use error::Error as EventLoopError;
pub use self::config::Config as EventLoopConfig;
#[derive(Debug)]
enum Message {
    Timeout(Option<Duration>),
    Wait(Socket, SocketEvents, usize),
}
const CURL_TOKEN: Token = Token(1);
const COMMUNICATOR_TOKEN: Token = Token(0);

/// Result of working event loop.
pub type EventLoopResult<T> = Result<T,error::Error>;
/// Event loop contains sockets, tokens, handlers.
pub struct EventLoop<T,E> {
    config: EventLoopConfig,
	poll: Poll,
	next_socket_token: usize,
	multi: Multi,
    #[allow(deprecated)]
	multi_socket_rx: mio::channel::Receiver<Message>,
    #[allow(deprecated)]
	multi_socket_tx: mio::channel::Sender<Message>,
	cur_timeout: Option<Duration>,
	socket_map: HashMap<usize, i32>,
    handle_map: HashMap<usize, Easy2Handle<T>>,
    next_handle_token: usize,
    #[allow(deprecated)]
    task_rx: mio::channel::Receiver<Task<T,E>>,
    result_tx: SyncSender<Result<Easy2<T>,TaskError<E>>>,
    executing_requests_handler: Option<Box<Fn(u32)>>,
    executing_requests: u32,
    running: bool,
}
impl <T,E>EventLoop<T,E> {
    #[allow(deprecated)]
    /// Creates new event loop.
	pub fn new(config: EventLoopConfig, task_rx: mio::channel::Receiver<Task<T,E>>,result_tx: SyncSender<Result<Easy2<T>,TaskError<E>>>) -> EventLoopResult<EventLoop<T,E>> {
		let (multi_socket_tx, multi_socket_rx) = mio::channel::channel();
		Ok(EventLoop {
            config: config,
			poll: Poll::new().map_err(|e| { error::Error::PollInit{ error: e } })?,
			next_socket_token: 2,
			multi: Multi::new(),
			multi_socket_rx: multi_socket_rx,
			multi_socket_tx: multi_socket_tx,
			cur_timeout: None,
            socket_map: HashMap::new(),
            handle_map: HashMap::new(),
            next_handle_token: 1,
            task_rx: task_rx,
            result_tx: result_tx,
            executing_requests_handler: None,
            executing_requests: 0,
            running: true,
		})
	}

    /// Callback passes count of executing requests.
    pub fn set_executing_requests_handler<F>(&mut self, handler: F) where F: Fn(u32) + 'static {
        self.executing_requests_handler = Some(Box::new(handler));
    }

    /// Returns count of active requests.
    pub fn get_active_requests(&self) -> u32 {
        return self.executing_requests;
    }

    fn init_multi(&mut self) -> EventLoopResult<()> {
        let multi_socket_tx = self.multi_socket_tx.clone();
        self.multi.socket_function(move |socket, events, token| {
            #[allow(deprecated)]
            multi_socket_tx.send(
                Message::Wait(socket, events, token)
            ).expect(
                "Unable to send socket"
            );
        }).map_err(
            |e| {
                error::Error::CurlMultiInitSocket {error: e}
            }
        )?;
        let multi_timeout_tx = self.multi_socket_tx.clone();
        self.multi.timer_function(move |dur| {
            #[allow(deprecated)]
            multi_timeout_tx.send(
                Message::Timeout(dur)
            ).expect(
                "Unable to send timeout"
            );
            true
        }).map_err(
            |e| {
                error::Error::CurlMultiInitTimeout {error: e}
            }
        )?;;
        Ok(())
    }

    fn init_poll(&mut self) -> EventLoopResult<()> {
        self.poll.register(
            &self.multi_socket_rx,
            CURL_TOKEN,
            #[allow(deprecated)]
            mio::Ready::all(),
            mio::PollOpt::level()
        ).map_err(|e|{
            error::Error::PollRegister {error: e}
        })?;
        self.poll.register(
            &self.task_rx,
            COMMUNICATOR_TOKEN,
            #[allow(deprecated)]
            mio::Ready::all(),
            mio::PollOpt::edge()
        ).map_err(|e|{
            error::Error::PollRegister {error: e}
        })?;
        Ok(())
    }

    /// Starts new event loop.
	pub fn start(&mut self) -> EventLoopResult<()> {
        self.init_multi()?;
        self.init_poll()?;

        let mut events = mio::Events::with_capacity(self.config.get_events_capacity());

        while self.running {
            let n = self.poll.poll(
                &mut events,
                self.cur_timeout
            ).map_err(|e|{
                return error::Error::PollPooling {error: e}
            })?;
            if n == 0 {
                let _ = self.multi.timeout().map_err(|e|{
                    error::Error::CurlMultiInitTimeout {error: e}
                })?;
            }

            for event in events.iter() {
                match event.token() {
                    CURL_TOKEN => {
                        self.process_multi_messages()?;
                    },
                    COMMUNICATOR_TOKEN => {
                        self.process_register_task()?;
                    },
                    _ => {
                        let _ = self.notify_multi_about_action(&event)?;
                    }
                }
            }
            self.process_result_data()?;
        }
        Ok(())
	}

    fn handle_executing_requests(&mut self, executing_count: u32) {
        if self.executing_requests != executing_count {
            self.executing_requests = executing_count;
            if let Some(ref callback) = self.executing_requests_handler {
                callback(executing_count);
            }
        }
    }

    fn register_task(&mut self, task: Task<T,E>) -> EventLoopResult<()> {
        if is_terminate_task(&task) {
            self.running = false;
            return Ok(());
        }
        match task.build() {
            Ok(handle) => {
                let mut e = self.multi.add2(
                    handle
                ).map_err(|e|{
                    error::Error::CurlMultiAdd {error: e}
                })?;
                let executing_count = self.multi.perform(
                ).map_err(|e|{
                    error::Error::CurlMultiPeroform {error: e}
                })?;
                self.handle_executing_requests(executing_count);
                e.set_token(self.next_handle_token).map_err(|e|{
                    error::Error::CurlSetToken {error: e}
                })?;
                if self.handle_map.insert(
                    self.next_handle_token,
                    e
                ).is_some() {
                    return Err(error::Error::UnableAddToken);
                }
                self.next_handle_token = self.next_handle_token.wrapping_add(1);
            },
            Err(error) => {
                self.result_tx.send(
                    Err(error)
                ).map_err(|_|{
                    error::Error::SendResultError
                })?;
            }
        }
        Ok(())
    }

    fn process_register_task(&mut self) -> EventLoopResult<()> {
        loop {
            #[allow(deprecated)]
            match self.task_rx.try_recv() {
                Ok(task) => {
                    self.register_task(task)?;
                },
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn process_multi_messages(&mut self) -> EventLoopResult<()> {
        loop {
            #[allow(deprecated)]
            match self.multi_socket_rx.try_recv() {
                Ok(Message::Timeout(dur)) => {
                    self.process_change_timeout(dur);
                },
                Ok(Message::Wait(socket, events, token)) => {
                    self.process_socket_function(socket, events, token)?;
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn process_socket_function(&mut self,socket: Socket, events: SocketEvents, token: usize) -> EventLoopResult<()> {
        let evented = mio::unix::EventedFd(&socket);
        if events.remove() {
            self.socket_map.remove(
                &token
            ).ok_or(error::Error::UnableRemoveToken)?;
            self.poll.deregister(
                &evented
            ).map_err(|e|{
                error::Error::PollReRegister {error: e}
            })?;
        } else {
            #[allow(deprecated)]
            let mut e = mio::Ready::none();
            if events.input() {
                e = e | mio::Ready::readable();
            }
            if events.output() {
                e = e | mio::Ready::writable();
            }
            if token == 0 {
                let token = self.next_socket_token;
                self.next_socket_token = self.next_socket_token.wrapping_add(1);
                self.multi.assign(
                    socket,
                    token
                ).map_err(|e|{
                    error::Error::CurlMultiAssign {error: e}
                })?;
                self.socket_map.insert(token, socket);
                self.poll.register(
                    &evented,
                    mio::Token(token),
                    e,
                    mio::PollOpt::level()
                ).map_err(|e|{
                    error::Error::PollRegister {error: e}
                })?;
            } else {
                self.poll.reregister(
                    &evented,
                    mio::Token(token),
                    e,
                    mio::PollOpt::level()
                ).map_err(|e|{
                    error::Error::PollReRegister {error: e}
                })?;
            }
        }
        Ok(())

    }

    fn process_change_timeout(&mut self, timeout: Option<Duration>) {
        self.cur_timeout = timeout;
    }

    #[allow(deprecated)]
    fn notify_multi_about_action(&mut self, event: &Event) -> EventLoopResult<u32> {
        let token = event.token();
        let socket = self.socket_map[&token.into()];
        let mut e = CurlEvents::new();
        if event.kind().is_readable() {
            e.input(true);
        }
        if event.kind().is_writable() {
            e.output(true);
        }
        if event.kind().is_error() {
            e.error(true);
        }
        let remaining = self.multi.action(
            socket, &e
        ).map_err(|e|{
            error::Error::CurlMultiAction {error: e}
        })?;
        Ok(remaining)
    }

    fn process_result_data(&mut self) -> EventLoopResult<()> {
        let mut messages = Vec::new();
        self.multi.messages(|message| {
            messages.push((message.result(),message.token()));
        });
        for (result,token) in messages.drain(..) {
            let token = token.map_err(|_|{
                error::Error::UnableRemoveToken
            })?;
            if let Some(result) = result {
                let handle = self.handle_map.remove(
                    &token
                ).ok_or(error::Error::UnableRemoveToken)?;
                let easy = self.multi.remove2(
                    handle
                ).map_err(|e|{
                    error::Error::CurlMultiRemove {error: e}
                })?;
                let result = result.map_err(|e|{
                    TaskError::Curl {error: e}
                });
                match result {
                    Ok(_) => {
                        self.result_tx.send(
                            Ok(easy)
                        ).map_err(|_|{
                            error::Error::SendResultError
                        })?;
                    },
                    Err(error) => {
                        self.result_tx.send(
                            Err(error)
                        ).map_err(|_|{
                            error::Error::SendResultError
                        })?;
                    },
                }
            }
        }
        Ok(())
    }
}

#[test]
fn event_loop_test() {
    use std::thread::spawn as thread_spawn;
    use std::io::Write;
    use std::sync::mpsc::sync_channel;
    use curl::easy::WriteError;
    use curl::easy::Handler as EasyHandler;
    struct Payload {
        data: Vec<u8>,
    };

    impl Payload {
        pub fn new() -> Payload {
            Payload {
                data: Vec::new(),
            }
        }
    }
    impl EasyHandler for Payload {
        fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
            self.data.write(data).unwrap();
            Ok(data.len())
        }
    }
    #[allow(deprecated)]
    let (tx_task,rx_task) = mio::channel::channel();
    let (tx_result,rx_result) = sync_channel(1000);

    thread_spawn(||{
        let mut worker: EventLoop<Payload,()> = EventLoop::new(
            EventLoopConfig::new(128), rx_task, tx_result
        ).unwrap();
        worker.start().unwrap();
    });

    #[allow(deprecated)]
    tx_task.send(Task::new(||{
        let mut easy =  Easy2::new(Payload::new());
        easy.url("http://google.com")?;
        easy.get(true)?;
        easy.follow_location(true)?;
        easy.timeout(Duration::from_secs(30))?;
        easy.max_connects(1)?;
        return Ok(easy);
    })).unwrap();

    #[allow(deprecated)]
    tx_task.send(Task::new(||{
        let mut easy =  Easy2::new(Payload::new());
        easy.url("http://github.com")?;
        easy.get(true)?;
        easy.follow_location(true)?;
        easy.timeout(Duration::from_secs(30))?;
        easy.max_connects(1)?;
        return Ok(easy);
    })).unwrap();

    let mut google = rx_result.recv().unwrap().unwrap();
    assert!(google.get_ref().data.len() >= 1);
    assert_eq!(google.response_code().unwrap(), 200);

    let mut github = rx_result.recv().unwrap().unwrap();
    assert!(github.get_ref().data.len() >= 1);
    assert_eq!(github.response_code().unwrap(), 200);
}