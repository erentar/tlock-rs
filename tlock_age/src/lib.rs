mod tle_age;

use anyhow::anyhow;

use std::{
    io::{copy, Read, Write},
    iter,
};
use tle_age::{HeaderIdentity, Identity, Recipient};

pub fn encrypt<W: Write, R: Read>(
    mut dst: W,
    mut src: R,
    chain_hash: &[u8],
    public_key_bytes: &[u8],
    round: u64,
) -> anyhow::Result<()> {
    let recipient = Recipient::new(chain_hash, public_key_bytes, round);
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(recipient)])
        .expect("we provided a recipient");

    let mut writer = encryptor.wrap_output(&mut dst).unwrap();
    copy(&mut src, &mut writer)?;
    writer.finish().unwrap();

    Ok(())
}

pub fn decrypt_round<R: Read>(src: R) -> anyhow::Result<u64> {
    let identity = HeaderIdentity::new();
    let decryptor = match age::Decryptor::new(src).unwrap() {
        age::Decryptor::Recipients(d) => d,
        _ => unreachable!(),
    };

    decryptor.decrypt(iter::once(&identity as &dyn age::Identity));
    match identity.round() {
        Some(round) => Ok(round),
        None => Err(anyhow!("Cannot decrypt round")),
    }
}

pub fn decrypt<W: Write, R: Read>(
    mut dst: W,
    src: R,
    chain_hash: &[u8],
    signature: &[u8],
) -> anyhow::Result<()> {
    let identity = Identity::new(chain_hash, signature);
    let decryptor = match age::Decryptor::new(src).unwrap() {
        age::Decryptor::Recipients(d) => d,
        _ => unreachable!(),
    };

    let mut reader = decryptor
        .decrypt(iter::once(&identity as &dyn age::Identity))
        .unwrap();
    copy(&mut reader, &mut dst)?;

    Ok(())
}
