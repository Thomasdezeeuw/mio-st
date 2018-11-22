use std::io::{self, Read, Write};
use std::net::{self, SocketAddr};
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::time::Duration;

use libc;
use net2::TcpStreamExt;

use crate::event::{Evented, EventedId, Ready};
use crate::poll::{PollCalled, PollOption, Poller};
use crate::sys::unix::eventedfd::EventedFd;

#[derive(Debug)]
pub struct TcpStream {
    stream: net::TcpStream,
}

impl TcpStream {
    pub fn connect(address: SocketAddr) -> io::Result<TcpStream> {
        // Create a raw socket descriptor.
        let socket_family = match address {
            SocketAddr::V4(..) => libc::AF_INET,
            SocketAddr::V6(..) => libc::AF_INET6,
        };
        let socket_fd = unsafe { libc::socket(socket_family, libc::SOCK_STREAM, 0) };
        if socket_fd == -1 {
            return Err(io::Error::last_os_error());
        }

        let stream = unsafe { net::TcpStream::from_raw_fd(socket_fd) };
        stream.set_nonblocking(true)?;

        match stream.connect(address) {
            Ok(..) => {},
            Err(ref e) if e.raw_os_error() == Some(libc::EINPROGRESS) => {},
            Err(e) => return Err(e),
        }

        Ok(TcpStream { stream })
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    pub fn set_ttl(&mut self, ttl: u32) -> io::Result<()> {
        self.stream.set_ttl(ttl)
    }

    pub fn ttl(&mut self) -> io::Result<u32> {
        self.stream.ttl()
    }

    pub fn set_keepalive(&self, keepalive: Option<Duration>) -> io::Result<()> {
        self.stream.set_keepalive(keepalive)
    }

    pub fn keepalive(&self) -> io::Result<Option<Duration>> {
        self.stream.keepalive()
    }

    pub fn set_nodelay(&mut self, nodelay: bool) -> io::Result<()> {
        self.stream.set_nodelay(nodelay)
    }

    pub fn nodelay(&mut self) -> io::Result<bool> {
        self.stream.nodelay()
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.peek(buf)
    }

    pub fn shutdown(&self, how: net::Shutdown) -> io::Result<()> {
        self.stream.shutdown(how)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.stream.take_error()
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl Evented for TcpStream {
    fn register(&mut self, poll: &mut Poller, id: EventedId, interests: Ready, opt: PollOption, p: PollCalled) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, id, interests, opt, p)
    }

    fn reregister(&mut self, poll: &mut Poller, id: EventedId, interests: Ready, opt: PollOption, p: PollCalled) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, id, interests, opt, p)
    }

    fn deregister(&mut self, poll: &mut Poller, p: PollCalled) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll, p)
    }
}

impl Into<net::TcpStream> for TcpStream {
    fn into(self) -> net::TcpStream {
        self.stream
    }
}

impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpStream {
        TcpStream {
            stream: net::TcpStream::from_raw_fd(fd),
        }
    }
}

impl IntoRawFd for TcpStream {
    fn into_raw_fd(self) -> RawFd {
        self.stream.into_raw_fd()
    }
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}

#[derive(Debug)]
pub struct TcpListener {
    listener: net::TcpListener,
}

impl TcpListener {
    pub fn new(address: SocketAddr) -> io::Result<TcpListener> {
        let listener = net::TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(TcpListener { listener })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (stream, address) = self.listener.accept()?;
        stream.set_nonblocking(true)?;
        Ok((TcpStream { stream }, address))
    }

    pub fn set_ttl(&mut self, ttl: u32) -> io::Result<()> {
        self.listener.set_ttl(ttl)
    }

    pub fn ttl(&mut self) -> io::Result<u32> {
        self.listener.ttl()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.listener.take_error()
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        self.listener.try_clone().map(|listener| TcpListener { listener })
    }
}

impl Evented for TcpListener {
    fn register(&mut self, poll: &mut Poller, id: EventedId, interests: Ready, opt: PollOption, p: PollCalled) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, id, interests, opt, p)
    }

    fn reregister(&mut self, poll: &mut Poller, id: EventedId, interests: Ready, opt: PollOption, p: PollCalled) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, id, interests, opt, p)
    }

    fn deregister(&mut self, poll: &mut Poller, p: PollCalled) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll, p)
    }
}

impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpListener {
        TcpListener {
            listener: net::TcpListener::from_raw_fd(fd),
        }
    }
}

impl IntoRawFd for TcpListener {
    fn into_raw_fd(self) -> RawFd {
        self.listener.into_raw_fd()
    }
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.listener.as_raw_fd()
    }
}
