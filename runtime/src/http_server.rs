use async_trait::async_trait;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::{
    convert::Infallible,
    future::Future,
    net::{IpAddr, SocketAddr},
};

#[async_trait]
trait Service {
    async fn init(&self, request: Request<Body>, ip: IpAddr) -> Result<Response<Body>, Infallible>;
}

pub async fn server<S>(port: u16, handler: S)
where
    S: Service + Sync,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let service = make_service_fn(move |conn: &AddrStream| {
        let ip = conn.remote_addr().ip();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| handler.init(req, ip.clone())));
        }
    });

    let server = Server::bind(&addr).serve(service);

    log::info!("Running on: {}", &addr.to_string());

    server.await.unwrap_or(());
}
