// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Types for the exported futures for the service endpoints.
//!
//! Each type is called by the code generated by [`wit-bindgen-guest-rust`] when the host calls the guest
//! Wasm module's respective endpoint. This module contains the code to forward the call to the
//! service type that implements [`linera-sdk::Service`].

use crate::{
    service::{system_api, wit_types},
    views::ViewStorageContext,
    Service, SimpleStateStorage, ViewStateStorage,
};
use async_trait::async_trait;
use linera_views::views::RootView;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

/// The storage APIs used by a service.
#[async_trait]
pub trait ServiceStateStorage {
    /// Loads the application state and run the given query.
    async fn handle_query(
        context: wit_types::QueryContext,
        argument: Vec<u8>,
    ) -> Result<Vec<u8>, String>;
}

#[async_trait]
impl<Application> ServiceStateStorage for SimpleStateStorage<Application>
where
    Application: Service + Default + DeserializeOwned + Serialize + Send + Sync,
{
    async fn handle_query(
        context: wit_types::QueryContext,
        argument: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let application: Arc<Application> = Arc::new(system_api::load().await);
        let argument: Application::Query =
            serde_json::from_slice(&argument).map_err(|e| e.to_string())?;
        let query_response = application
            .handle_query(&context.into(), argument)
            .await
            .map_err(|error| error.to_string())?;
        serde_json::to_vec(&query_response).map_err(|e| e.to_string())
    }
}

#[async_trait]
impl<Application> ServiceStateStorage for ViewStateStorage<Application>
where
    Application: Service + RootView<ViewStorageContext> + Send + Sync,
    Application::Error: Send,
{
    async fn handle_query(
        context: wit_types::QueryContext,
        argument: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let application: Arc<Application> = Arc::new(system_api::lock_and_load_view().await);
        let argument: Application::Query =
            serde_json::from_slice(&argument).map_err(|e| e.to_string())?;
        let result = application.handle_query(&context.into(), argument).await;
        if result.is_ok() {
            system_api::unlock_view().await;
        }
        let query_response = result.map_err(|error| error.to_string())?;
        serde_json::to_vec(&query_response).map_err(|e| e.to_string())
    }
}