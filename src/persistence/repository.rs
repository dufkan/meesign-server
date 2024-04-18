use uuid::Uuid;

use super::{
    enums::{KeyType, ProtocolType, TaskType},
    error::PersistenceError,
    models::{Device, Group, Task},
};

use self::device::{activate_device, add_device, get_devices};
use self::group::{add_group, get_groups};
use self::task::create_task;

use diesel::{Connection, PgConnection};
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use std::sync::Arc;

mod device;
mod group;
mod task;
mod utils;

pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Repository {
    pg_pool: Arc<PgPool>,
}

type PgPool = Pool<AsyncPgConnection>;
type DeadpoolPgConnection = Object<AsyncPgConnection>;

impl Repository {
    pub async fn from_url(database_url: &str) -> Result<Self, PersistenceError> {
        let repo = Self {
            pg_pool: Arc::new(Repository::init_pool(database_url)?),
        };
        repo.apply_migrations()?;
        Ok(repo)
    }

    pub fn apply_migrations(&self) -> Result<(), PersistenceError> {
        // TODO: can we do it in async?
        let mut conn = self.get_connection()?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Couldn't apply migrations");
        Ok(())
    }

    fn init_pool(database_url: &str) -> Result<PgPool, PersistenceError> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        Ok(Pool::builder(config).build()?)
    }

    async fn get_async_connection(&self) -> Result<DeadpoolPgConnection, PersistenceError> {
        Ok(self.pg_pool.get().await.unwrap())
    }

    fn get_connection(&self) -> Result<PgConnection, PersistenceError> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Ok(PgConnection::establish(&database_url).unwrap())
    }
}

/// Interface
impl Repository {
    /* Devices */
    pub async fn add_device(
        &self,
        identifier: &[u8],
        name: &str,
        certificate: &[u8],
    ) -> Result<Device, PersistenceError> {
        add_device(
            &mut self.get_async_connection().await?,
            identifier,
            name,
            certificate,
        )
        .await
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>, PersistenceError> {
        get_devices(&mut self.get_async_connection().await?).await
    }

    pub async fn get_task_devices(&self, task_id: &Uuid) -> Result<Vec<Device>, PersistenceError> {
        todo!()
    }

    pub async fn activate_device(
        &self,
        target_identifier: &[u8],
    ) -> Result<Option<Device>, PersistenceError> {
        activate_device(&mut self.get_async_connection().await?, target_identifier).await
    }

    /* Groups */
    pub async fn add_group<'a>(
        &self,
        identifier: &[u8],
        name: &str,
        devices: &[&[u8]],
        threshold: u32,
        protocol: ProtocolType,
        certificate: Option<&[u8]>,
    ) -> Result<Group, PersistenceError> {
        add_group(
            &mut self.get_async_connection().await?,
            identifier,
            name,
            devices,
            threshold,
            protocol,
            certificate,
        )
        .await
    }

    pub async fn get_group(
        &self,
        group_identifier: &Vec<u8>,
    ) -> Result<Option<Group>, PersistenceError> {
        todo!()
    }

    pub async fn get_groups(&self) -> Result<Vec<Group>, PersistenceError> {
        get_groups(&mut self.get_async_connection().await?).await
    }

    pub async fn get_device_groups(
        &self,
        identifier: &[u8],
    ) -> Result<Vec<Group>, PersistenceError> {
        todo!()
    }

    pub async fn does_group_contain_device(
        &self,
        group_id: &[u8],
        device_id: &[u8],
    ) -> Result<bool, PersistenceError> {
        todo!()
    }

    /* Tasks */
    pub async fn create_group_task<'a>(
        &self,
        name: &str,
        devices: &[Vec<u8>],
        threshold: u32,
        protocol_type: ProtocolType,
        key_type: KeyType,
    ) -> Result<Task, PersistenceError> {
        create_task(
            &mut self.get_async_connection().await?,
            TaskType::Group,
            name,
            None,
            devices,
            Some(threshold),
            Some(key_type),
            Some(protocol_type),
        )
        .await
    }

    pub async fn create_sign_task<'a>(
        &self,
        group_identifier: &Vec<u8>,
        name: &str,
        data: &Vec<u8>,
    ) -> Result<Task, PersistenceError> {
        create_task(
            &mut self.get_async_connection().await?,
            TaskType::SignChallenge, // TODO: based on data
            name,
            Some(data),
            &vec![],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn create_decrypt_task(
        &self,
        group_identifier: &Vec<u8>,
        name: &str,
        data: &Vec<u8>,
    ) -> Result<Task, PersistenceError> {
        create_task(
            &mut self.get_async_connection().await?,
            TaskType::Decrypt,
            name,
            Some(data),
            &vec![],
            None,
            None,
            None,
        )
        .await
    }

    pub async fn get_task(&self, task_id: &Uuid) -> Result<Option<Task>, PersistenceError> {
        todo!()
    }

    pub async fn get_tasks(&self) -> Result<Vec<Task>, PersistenceError> {
        todo!()
    }

    pub async fn get_tasks_for_restart(&self) -> Result<Vec<Task>, PersistenceError> {
        todo!()
    }

    pub async fn get_device_tasks(&self, identifier: &[u8]) -> Result<Vec<Task>, PersistenceError> {
        todo!()
    }
}