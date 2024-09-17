use std::cell::RefCell;
use std::rc::Rc;
use std::{any, io};

use folo::net::TcpConnection as TcpStream;
use ntex_bytes::BytesVec;
use ntex_io::{
    Handle, IoStream, ReadContext, WriteContext, WriteContextBuf,
};

impl IoStream for crate::TcpStream {
    fn start(self, read: ReadContext, write: WriteContext) -> Option<Box<dyn Handle>> {
        let io = Rc::new(RefCell::new(self.0));

        let mut rio = Read(io.clone());
        folo::rt::spawn(async move {
            read.handle(&mut rio).await;
        });
        let mut wio = Write(io.clone());
        folo::rt::spawn(async move {
            write.handle(&mut wio).await;
        });
        Some(Box::new(HandleWrapper(io)))
    }
}

#[allow(dead_code)]
struct HandleWrapper(Rc<RefCell<TcpStream>>);

impl Handle for HandleWrapper {
    fn query(&self, _id: any::TypeId) -> Option<Box<dyn any::Any>> {
        /*if id == any::TypeId::of::<types::PeerAddr>() {
            if let Ok(addr) = self.0.borrow().peer_addr() {
                return Some(Box::new(types::PeerAddr(addr)));
            }
        } else if id == any::TypeId::of::<SocketOptions>() {
            return Some(Box::new(SocketOptions(Rc::downgrade(&self.0))));
        }*/
        None
    }
}

/// Read io task
struct Read(Rc<RefCell<TcpStream>>);

impl ntex_io::AsyncRead for Read {
    #[inline]
    async fn read(&mut self, mut buf: BytesVec) -> (BytesVec, io::Result<usize>) {
        use folo::io::OperationResultExt;

        let buf_ptr = buf.as_mut_ptr();
        let capacity = buf.capacity();
        let pinned_buf = unsafe { folo::io::PinnedBuffer::from_ptr(buf_ptr, capacity) };

        let result = self.0.borrow_mut().receive(pinned_buf).await.into_inner();

        match result {
            Ok(pinned_buf) => {
                (buf, Ok(pinned_buf.len()))
            }
            Err(e) => {
                (buf, Err(to_std_io_error(e)))
            }
        }
    }
}

struct Write(Rc<RefCell<TcpStream>>);

impl ntex_io::AsyncWrite for Write {
    #[inline]
    async fn write(&mut self, buf: &mut WriteContextBuf) -> io::Result<()> {
        use folo::io::OperationResultExt;

        let buf = buf.take();
        if buf.is_none() {
            return Ok(());
        }

        let mut buf = buf.unwrap();
        let buf_ptr = buf.as_mut_ptr();
        let len = buf.len();
        let pinned_buf = unsafe { folo::io::PinnedBuffer::from_ptr(buf_ptr, len) };

        let result = self.0.borrow_mut().send(pinned_buf).await.into_inner();
        match result {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(to_std_io_error(e))
            }
        }
    }

    #[inline]
    async fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    async fn shutdown(&mut self) -> io::Result<()> {
        let result = self.0.borrow_mut().shutdown().await;
        to_std_io_result(result)
    }
}

fn to_std_io_result<T>(r: folo::io::Result<T>) -> std::io::Result<T> {
    match r {
        Ok(t) => Ok(t),
        Err(e) => Err(to_std_io_error(e)),
    }
}

fn to_std_io_error(e: folo::io::Error) -> std::io::Error {
    if let folo::io::Error::StdIo(e) = e {
        e
    } else {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{e}"))
    }
}
