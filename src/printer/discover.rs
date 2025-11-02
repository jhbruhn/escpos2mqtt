use snmp2::{AsyncSession, Oid, Value};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use tokio::net::UdpSocket;

const DISCOVERY_RESPONSE_TIMEOUT: Duration = Duration::from_millis(100);
const SNMP_RESPONSE_TIMEOUT: Duration = Duration::from_millis(100);
const DEFAULT_PORT: u16 = 9100;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error")]
    IoError(#[from] std::io::Error),

    #[error("SNMP error")]
    SnmpError(#[from] snmp2::Error),

    #[error("timeout")]
    TimeoutError,
    #[error("no description")]
    NoDescription,
    #[error("no name")]
    NoName,
}

#[derive(Debug)]
pub struct Info {
    pub name: String,
    pub description: String,
    pub address: SocketAddr,
}

async fn get_snmp_details(addr: &IpAddr) -> Result<Info, Error> {
    // timeouts should be handled by the caller with `tokio::time::timeout`
    let sys_name_oid = Oid::from(&[1, 3, 6, 1, 2, 1, 1, 5, 0]).unwrap();
    let sys_descr_oid = Oid::from(&[1, 3, 6, 1, 2, 1, 1, 1, 0]).unwrap();
    let community = b"public";
    let mut sess = AsyncSession::new_v1(SocketAddr::new(addr.clone(), 161), community, 0)
        .await
        .map_err(Error::IoError)?;
    let mut response = sess.get(&sys_descr_oid).await.unwrap();
    let description = if let Some((_oid, Value::OctetString(sys_descr))) = response.varbinds.next()
    {
        let value = String::from_utf8_lossy(sys_descr);
        Ok(value.into_owned())
    } else {
        Err(Error::NoDescription)
    }?;
    let mut response = sess.get(&sys_name_oid).await.unwrap();
    let name = if let Some((_oid, Value::OctetString(sys_descr))) = response.varbinds.next() {
        let value = String::from_utf8_lossy(sys_descr);
        Ok(value.into_owned())
    } else {
        Err(Error::NoName)
    }?;

    let address = SocketAddr::new(addr.clone(), DEFAULT_PORT); // todo: discover port?

    Ok(Info {
        name,
        description,
        address,
    })
}

pub async fn discover_network_printers() -> Result<Vec<Info>, Error> {
    log::debug!("discover_network_printers: binding UDP socket");
    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    sock.set_broadcast(true)?;

    let query = "EPSONP";
    let query: Vec<u8> = query
        .as_bytes()
        .iter()
        .chain([0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00].iter())
        .copied()
        .collect();

    log::debug!("discover_network_printers: sending broadcast");
    let _len = sock
        .send_to(
            &query,
            "255.255.255.255:3289"
                .parse::<std::net::SocketAddr>()
                .unwrap(),
        )
        .await?;

    let mut printers = vec![];
    const MAX_RESPONSES: usize = 100; // Safety limit

    log::debug!("discover_network_printers: waiting for responses");
    loop {
        if printers.len() >= MAX_RESPONSES {
            log::warn!("Reached maximum discovery responses ({}), stopping", MAX_RESPONSES);
            break;
        }

        let mut buf = [0 as u8; 1024];
        if let Ok(Ok((_len, addr))) =
            tokio::time::timeout(DISCOVERY_RESPONSE_TIMEOUT, sock.recv_from(&mut buf)).await
        {
            log::debug!("discover_network_printers: got response from {}", addr);

            // Spawn SNMP query in separate task to avoid stack overflow
            let ip = addr.ip();
            let handle = tokio::spawn(async move {
                get_snmp_details(&ip).await
            });

            if let Ok(Ok(Ok(info))) =
                tokio::time::timeout(SNMP_RESPONSE_TIMEOUT, handle).await
            {
                log::debug!("discover_network_printers: got SNMP info: {:?}", info);
                printers.push(info);
            } else {
                log::debug!("discover_network_printers: SNMP timeout or error for {}", addr);
            }
        } else {
            log::debug!("discover_network_printers: no more responses");
            break;
        }
    }

    log::debug!("discover_network_printers: returning {} printers", printers.len());
    Ok(printers)
}
