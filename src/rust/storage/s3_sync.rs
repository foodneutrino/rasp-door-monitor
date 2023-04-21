use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::error::{ListBucketsError, ListObjectsV2Error};
use aws_sdk_s3::output::{ListBucketsOutput, ListObjectsV2Output, PutObjectOutput};
use aws_sdk_s3::types::{ByteStream, SdkError};
use aws_sdk_s3::{Client, Region};
use std::path::{Path, PathBuf};
use super::base::StorageEngine;
use tokio::runtime::Runtime;

pub struct S3BlockingClient {
    client: aws_sdk_s3::Client,

    /// A `current_thread` runtime for executing operations on the
    /// asynchronous client in a blocking manner.
    rt: Runtime,

    bucket_name: String,
}

impl S3BlockingClient {
    pub fn get_buckets(&mut self) -> Result<ListBucketsOutput, SdkError<ListBucketsError>> {
        // pub fn get_buckets(&mut self) -> Result<ListBucketsOutput> {
        self.rt.block_on(self.client.list_buckets().send())
    }

    pub fn list_bucket_objects(
        &mut self,
        bucket_name: String,
    ) -> Result<ListObjectsV2Output, SdkError<ListObjectsV2Error>> {
        // ) -> Result<ListObjectsV2Output> {
        self.rt
            .block_on(self.client.list_objects_v2().bucket(bucket_name).send())
    }

    pub fn set_destination_bucket(&mut self, bucket_name: String) {
        self.bucket_name = bucket_name;
    }

    pub fn store(
        &self,
        source_file: &Path,
        destination_bucket: &str
    ) -> Result<PutObjectOutput, String> {
        println!(
            "Attemp to store {} at {}",
            source_file.to_str().unwrap_or("no-file-here"),
            destination_bucket
        );
        self.rt
            .block_on(ByteStream::from_path(source_file))
            .map_err(|e| e.to_string())
            .and_then(|body| {
                Ok(self
                    .client
                    .put_object()
                    .bucket(destination_bucket)
                    .key(source_file.to_str().unwrap())
                    .body(body)
                )
            })
            .and_then(|put_obj_future| {
                self.rt
                    .block_on(put_obj_future.send())
                    //.map_err(|_| "S3 store failed")
                    .map_err(|err| err.to_string())
            })
    }
}

pub fn connect(region: &str) -> Result<S3BlockingClient, &'static str> {
    // pub fn connect(region: String) -> Result<S3BlockingClient> {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => {
            // `connect` to sync version of aws client

            // set up aws env
            let region_provider = RegionProviderChain::first_try(Region::new(String::from(region)))
                .or_default_provider()
                .or_else(Region::new("us-east-1"));

            if let Some(set_region) = rt.block_on(region_provider.region()) {
                println!("Region [{}]", set_region);
            }

            let shared_config = rt.block_on(aws_config::from_env().region(region_provider).load());
            let client = Client::new(&shared_config);

            let bucket_name = String::from("door-images");

            Ok(S3BlockingClient { client, rt, bucket_name})
        }
        Err(_) => Err("Failed creating async runtime"),
    }
}

impl StorageEngine for S3BlockingClient {
    fn create_destination(&mut self, identifier: &str) -> Result<Box<Path>, &'static str> {
        self.set_destination_bucket(String::from(identifier));
        // this doesn't really make sense - why return a local path for a remotely created one
        let mut ret_path = PathBuf::new();
        ret_path.set_file_name(identifier);
        Ok(ret_path.into_boxed_path())
    }

    fn store(&self, local_path: &str, destination: &str) -> Result<(), String> {
        println!("Writing {} to s3 {}", local_path, destination); // need to add bucket!
        let mut store_file = PathBuf::new();
        store_file.set_file_name(local_path);
        if let Err(err_str) = self.store(
            store_file.as_path(), 
            format!({}/{}, self.bucket_name, destination).as_str()
        ) {
            Err(format!("move {} to S3 Failed: {}", local_path, err_str))
        } else {
            Ok(())
        }
    }
}

unsafe impl Send for S3BlockingClient {}
unsafe impl Sync for S3BlockingClient {}
