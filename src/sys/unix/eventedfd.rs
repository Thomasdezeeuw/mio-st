use std::io;
use std::os::unix::io::RawFd;

use event::{EventedId, Evented};
use poll::{Poll, PollOpt, Ready};

/// Adapter for a `RawFd` providing an [`Evented`] implementation.
///
/// `EventedFd` enables registering any type with an file descriptor with
/// [`Poll`].
///
/// While only implementations for TCP and UDP are provided, Mio supports
/// registering any file descriptor that can be registered with the underlying
/// OS selector. `EventedFd` provides the necessary bridge.
///
/// Note that `EventedFd` takes a reference to a `RawFd`. This is because
/// `EventedFd` **does not** take ownership of the file descriptor.
/// Specifically, it will not manage any lifecycle related operations, such as
/// closing the file descriptor on drop. It is expected that the `EventedFd` is
/// constructed right before a call to [`Poll.register`]. See the examples
/// below for more detail.
///
/// [`Evented`]: ../event/trait.Evented.html
/// [`Poll`]: ../struct.Poll.html
/// [`Poll.register`]: ../struct.Poll.html#method.register
///
/// # Examples
///
/// Basic usage
///
/// ```
/// # use std::error::Error;
/// # fn try_main() -> Result<(), Box<Error>> {
/// use std::net::TcpListener;
/// use std::os::unix::io::AsRawFd;
///
/// use mio::event::{Evented, EventedId};
/// use mio::poll::{Poll, PollOpt, Ready};
/// use mio::unix::EventedFd;
///
/// // Bind a listener from the standard library.
/// let listener = TcpListener::bind("127.0.0.1:0")?;
///
/// let mut poll = Poll::new()?;
///
/// // Register the listener using `EventedFd`.
/// poll.register(&mut EventedFd(&listener.as_raw_fd()), EventedId(0), Ready::READABLE, PollOpt::EDGE)?;
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
/// ```
///
/// Implementing `Evented` for a custom type backed by a `RawFd`.
///
/// ```
/// use std::io;
/// use std::os::unix::io::RawFd;
///
/// use mio::event::{Evented, EventedId};
/// use mio::poll::{Poll, PollOpt, Ready};
/// use mio::unix::EventedFd;
///
/// pub struct MyIo {
///     fd: RawFd,
/// }
///
/// impl Evented for MyIo {
///     fn register(&mut self, poll: &mut Poll, id: EventedId, interests: Ready, opt: PollOpt) -> io::Result<()> {
///         EventedFd(&self.fd).register(poll, id, interests, opt)
///     }
///
///     fn reregister(&mut self, poll: &mut Poll, id: EventedId, interests: Ready, opt: PollOpt) -> io::Result<()> {
///         EventedFd(&self.fd).reregister(poll, id, interests, opt)
///     }
///
///     fn deregister(&mut self, poll: &mut Poll) -> io::Result<()> {
///         EventedFd(&self.fd).deregister(poll)
///     }
/// }
/// ```
#[derive(Debug)]
pub struct EventedFd<'a>(pub &'a RawFd);

impl<'a> Evented for EventedFd<'a> {
    fn register(&mut self, poll: &mut Poll, id: EventedId, interests: Ready, opt: PollOpt) -> io::Result<()> {
        poll.selector().register(*self.0, id, interests, opt)
    }

    fn reregister(&mut self, poll: &mut Poll, id: EventedId, interests: Ready, opt: PollOpt) -> io::Result<()> {
        poll.selector().reregister(*self.0, id, interests, opt)
    }

    fn deregister(&mut self, poll: &mut Poll) -> io::Result<()> {
        poll.selector().deregister(*self.0)
    }
}
