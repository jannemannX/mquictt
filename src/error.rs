#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io Error : {0}")]
    Io(#[from] std::io::Error),
    #[error("Endpoint Error : {0}")]
    Endpoint(#[from] quinn::EndpointError),
    #[error("Server Connect Error : {0}")]
    Connection(#[from] quinn::ConnectionError),
    #[error("Client Connect Error : {0}")]
    Connect(#[from] quinn::ConnectError),
    #[error("Write Error : {0}")]
    Write(#[from] quinn::WriteError),
    #[error("Read Error : {0}")]
    Read(#[from] quinn::ReadError),
    #[error("Connection broken")]
    ConnectionBroken,
    #[error("MQTT Error : {0}")]
    MQTT(mqttbytes::Error),
    #[error("Rustls Error : {0}")]
    Rustls(#[from] rustls::TLSError),
    #[error("Missing Tls Certificate Error")]
    MissingCertificate,
    #[error("Sub Request Tx Error : {0}")]
    PubDataTx(#[from] flume::SendError<mqttbytes::v4::Publish>),
    #[error("Pub Data Recv Error : {0}")]
    PubDataRx(#[from] flume::RecvError),
    #[error("Sub Request Tx Error : {0}")]
    SubReqTx(#[from] flume::SendError<flume::Sender<mqttbytes::v4::Publish>>),
    #[error("Sub Request Tx Error : {0}")]
    SubReqRx(flume::RecvError),
    #[error("Config Parse Error : {0}")]
    ConfigParse(#[from] serde_json::Error),
}
