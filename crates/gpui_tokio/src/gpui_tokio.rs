use std::future::Future;

use gpui::{App, AppContext, Global, ReadGlobal, Task};

/// Simple defer implementation - executes a closure when dropped
struct Defer<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

fn defer<F: FnOnce()>(f: F) -> Defer<F> {
    Defer(Some(f))
}

pub use tokio::task::JoinError;

pub fn init(cx: &mut App) {
    cx.set_global(GlobalTokio::new());
}

struct GlobalTokio {
    runtime: tokio::runtime::Runtime,
}

impl Global for GlobalTokio {}

impl GlobalTokio {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            // Since we now have two executors, let's try to keep our footprint small
            .worker_threads(2)
            // A chrome-mode fetch is one poll chain through spider's navigation
            // futures, and those are large async state machines nested dozens
            // deep. Unoptimized they do not fit in tokio's 2 MiB default: the
            // worker overflows its stack partway into the first page, which
            // aborts the process rather than failing the crawl. The stack is
            // reserved address space, touched only as deep as it is used, so
            // the headroom costs nothing in practice.
            .thread_stack_size(16 * 1024 * 1024)
            .enable_all()
            .build()
            .expect("Failed to initialize Tokio");

        Self { runtime }
    }
}

pub struct Tokio {}

impl Tokio {
    /// Spawns the given future on Tokio's thread pool, and returns it via a GPUI task
    /// Note that the Tokio task will be cancelled if the GPUI task is dropped
    pub fn spawn<C, Fut, R>(cx: &C, f: Fut) -> Task<Result<R, JoinError>>
    where
        C: AppContext,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.runtime.spawn(f);
            let abort_handle = join_handle.abort_handle();
            let cancel = defer(move || {
                abort_handle.abort();
            });
            cx.background_spawn(async move {
                let result = join_handle.await;
                drop(cancel);
                result
            })
        })
    }

    /// Spawns the given future on Tokio's thread pool, and returns it via a GPUI task
    /// Note that the Tokio task will be cancelled if the GPUI task is dropped
    pub fn spawn_result<C, Fut, R>(cx: &C, f: Fut) -> Task<anyhow::Result<R>>
    where
        C: AppContext,
        Fut: Future<Output = anyhow::Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.runtime.spawn(f);
            let abort_handle = join_handle.abort_handle();
            let cancel = defer(move || {
                abort_handle.abort();
            });
            cx.background_spawn(async move {
                let result = join_handle.await?;
                drop(cancel);
                result
            })
        })
    }

    pub fn handle(cx: &App) -> tokio::runtime::Handle {
        GlobalTokio::global(cx).runtime.handle().clone()
    }
}
