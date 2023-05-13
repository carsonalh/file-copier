use std::fs;

use opendal::services;
use opendal::Operator;
use opendal::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct SftpConnection {
    host: String,
    user: String,
    key: String,
}

#[derive(Deserialize)]
struct S3Connection {
    region: String,
    bucket: String,
}

#[derive(Deserialize)]
enum Source {
    Sftp(SftpConnection),
    S3(S3Connection),
}

#[derive(Deserialize)]
enum Destination {
    Sftp(SftpConnection),
    S3(S3Connection),
}

#[derive(Deserialize)]
struct Job {
    from: Source,
    to: Destination,
    file: String,
}

#[derive(Deserialize)]
struct Configuration {
    jobs: Vec<Job>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let contents = fs::read_to_string("configuration.yml").unwrap();

    let configuration: Configuration = serde_yaml::from_str(&contents).unwrap();

    for job in configuration.jobs {
        let source_op = match job.from {
            Source::Sftp(SftpConnection { host, user, key }) => {
                let mut sftp_builder = services::Sftp::default();

                sftp_builder
                    .endpoint(&host)
                    .user(&user)
                    .key(&key);

                Operator::new(sftp_builder)?.finish()
            },
            Source::S3(S3Connection { region, bucket }) => {
                let mut s3_builder = services::S3::default();

                s3_builder
                    .region(&region)
                    .bucket(&bucket);

                Operator::new(s3_builder)?.finish()
            },
        };

        let destination_op = match job.to {
            Destination::Sftp(SftpConnection { host, user, key }) => {
                let mut sftp_builder = services::Sftp::default();

                sftp_builder
                    .endpoint(&host)
                    .user(&user)
                    .key(&key);

                Operator::new(sftp_builder)?.finish()
            },
            Destination::S3(S3Connection { region, bucket }) => {
                let mut s3_builder = services::S3::default();

                s3_builder
                    .region(&region)
                    .bucket(&bucket);

                Operator::new(s3_builder)?.finish()
            },
        };

        let bs = source_op.read(&job.file).await?;

        destination_op.write(&job.file, bs).await?;
    }

    Ok(())
}