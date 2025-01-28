use clap::{arg, command, Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};
use hex::{decode, encode};
use auth_rs::{AuthorityCertificate, AuthorityCertificateBuilder};

#[derive(Parser, Debug)]
#[command()]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Create {
        #[arg(long)]
        certifier_private_key: String,
        #[arg(long)]
        certified_public_key: String,
    },
    Verify {
        #[arg(long)]
        certificate: String,
        #[arg(long)]
        certifier_public_key: String,
        #[arg(long)]
        certified_public_key: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            certifier_private_key,
            certified_public_key,
        } => {
            let certifier_key_bytes = decode(certifier_private_key)?;
            let certified_key_bytes = decode(certified_public_key)?;
            let certifier_signing_key = SigningKey::try_from(certifier_key_bytes.as_slice())?;
            let certified_verifying_key = VerifyingKey::try_from(certified_key_bytes.as_slice())?;
            let builder = AuthorityCertificateBuilder::default();
            let certificate = builder
                .for_authority(certified_verifying_key)
                .from_certifier(certifier_signing_key)
                .build();

            let serialized = certificate.serialize_protobuf();
            let hex_encoded = encode(serialized);

            println!("certificate created:\n{}", hex_encoded);
        }
        Commands::Verify {
            certificate,
            certifier_public_key,
            certified_public_key,
        } => {
            let cert_bytes = decode(certificate)?;
            let certificate = AuthorityCertificate::try_from(cert_bytes.as_slice())?;
            match certificate.verify(certified_public_key, certifier_public_key) {
                Ok(_) => println!("certificate is valid"),
                Err(e) => println!("certificate is invalid: {}", e),
            }
        }
    }

    Ok(())
}
