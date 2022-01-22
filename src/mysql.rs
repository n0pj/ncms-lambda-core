use aws_sdk_s3::{Client, Region};
use diesel::prelude::*;

use diesel::sql_query;
use ncms_core::db::mysql::establish_connection;
use regex::Regex;
use rusoto_core::Region as RusotoRegion;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use std::env;
use std::io::prelude::*;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
type Migrations = Vec<String>;
type MigrationKeys = Vec<String>;
type RawSqls = Vec<RawSql>;

// struct Mysql;

// impl Mysql {
//     fn establish_connection() {}
// }

struct RawSql {
    sql: String,
    key: String,
}

pub struct Migration {
    bucket: String,
    region: String,
    // migration_keys: Option<MigrationKeys>,
    // migrations: Option<Migrations>,
    // up_migrations: Option<Migrations>,
    // raw_sqls: Option<RawSqls>,
}

impl Default for Migration {
    fn default() -> Self {
        Self {
            bucket: "".to_owned(),
            region: "".to_owned(),
            // migration_keys: None,
            // migrations: None,
            // up_migrations: None,
            // raw_sqls: None,
        }
    }
}

impl Migration {
    pub fn new(bucket: &str, region: &str) -> Self {
        Self {
            bucket: bucket.to_owned(),
            region: region.to_owned(),
            ..Default::default()
        }
    }

    ///
    /// S3 からマイグレーションを取得する
    ///
    async fn get_migration_keys(&self) -> Result<Vec<String>, Error> {
        let shared_config = aws_config::from_env()
            .region(Region::new(self.region.clone()))
            .load()
            .await;
        let client = Client::new(&shared_config);
        let res = client.list_objects_v2().bucket(&self.bucket).send().await?;
        let mut migrations = vec![];

        for object in res.contents().unwrap_or_default() {
            let key = object.key().unwrap_or_default();

            migrations.push(key.to_owned())
        }

        Ok(migrations)
    }

    // fn graphiql() -> String {
    //     graphiql_source("/graphqk", None)
    // }

    ///
    /// S3 から SQL を取得する
    ///
    async fn get_sql(&self, key: String) -> Result<String, Error> {
        let client = S3Client::new(RusotoRegion::ApNortheast1);
        let bucket = env::var("S3_MIGRATIONS_BUCKET").expect("get_migration_keys error");
        let object = client
            .get_object(GetObjectRequest {
                bucket,
                key,
                ..Default::default()
            })
            .await?;
        let sql = tokio::task::spawn_blocking(|| {
            let mut stream = object.body.unwrap().into_blocking_read();
            let mut body = String::new();

            stream.read_to_string(&mut body).unwrap();

            // println!("{:?}", body);
            body
        })
        .await?;

        Ok(sql)
    }

    ///
    /// key を元に S3 から SQL を取得する
    ///
    async fn get_raw_sqls(&self, migration_keys: Migrations) -> Result<RawSqls, Error> {
        let mut raw_sqls = vec![];

        for key in migration_keys {
            let key = key.to_owned();
            let migration = RawSql {
                key: key.clone(),
                sql: self.get_sql(key).await?,
            };

            raw_sqls.push(migration);
        }

        Ok(raw_sqls)
    }

    ///
    /// Migrations から down.sql を抽出する
    ///
    fn get_down_migrations(&self, migration_keys: &Migrations) -> Migrations {
        let mut down_migrations = vec![];

        for key in migration_keys {
            let re = Regex::new(r"^migrations.*down\.sql$").unwrap();

            if re.is_match(&key) {
                down_migrations.push(key.to_owned())
            }
        }

        down_migrations
    }

    ///
    /// Migrations から up.sql を抽出する
    ///
    fn get_up_migrations(&self, migration_keys: &MigrationKeys) -> Migrations {
        let mut up_migrations = vec![];

        for key in migration_keys {
            let re = Regex::new(r"^migrations.*up\.sql$").unwrap();

            if re.is_match(&key) {
                up_migrations.push(key.to_owned())
            }
        }

        up_migrations
    }

    ///
    /// down.sql を実行する。
    /// 空の SQL はスキップされる。
    /// また、コメント付き SQL はエラーとなる。
    ///
    pub async fn execute_down_migrations(self) -> Result<bool, Error> {
        let migration_keys = self.get_migration_keys().await?;
        let down_migrations = self.get_down_migrations(&migration_keys);
        let raw_sqls = self.get_raw_sqls(down_migrations).await?;
        let conn = establish_connection();

        for raw_sql in raw_sqls {
            let raw_sql_len = raw_sql.sql.len();
            let sql = raw_sql.sql;

            // 空の SQL をスキップ
            if raw_sql_len == 0 {
                continue;
            };

            let result = sql_query(&sql).execute(&conn);

            match result {
                Ok(_) => println!("{} executed", raw_sql.key),
                Err(e) => {
                    println!("key: \n{}", raw_sql.key);
                    println!("sql: \n{}", sql);

                    panic!("Migration error: {:?}", e)
                }
            }
        }

        Ok(true)
    }

    ///
    /// up.sql を実行する。
    /// 空の SQL はスキップされる。
    /// また、コメント付き SQL はエラーとなる。
    ///
    pub async fn execute_up_migrations(self) -> Result<bool, Error> {
        let migration_keys = self.get_migration_keys().await?;
        let up_migrations = self.get_up_migrations(&migration_keys);
        let raw_sqls = self.get_raw_sqls(up_migrations).await?;
        let conn = establish_connection();

        for raw_sql in raw_sqls {
            let raw_sql_len = raw_sql.sql.len();
            let sql = raw_sql.sql;

            // 空の SQL をスキップ
            if raw_sql_len == 0 {
                continue;
            };

            let result = sql_query(&sql).execute(&conn);

            match result {
                Ok(_) => println!("{} executed", raw_sql.key),
                Err(e) => {
                    println!("key: \n{}", raw_sql.key);
                    println!("sql: \n{}", sql);

                    panic!("Migration error: {:?}", e)
                }
            }
        }

        Ok(true)
    }
}
