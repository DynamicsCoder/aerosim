use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use pyo3::prelude::*;
use r2r::{Node, Publisher, Subscriber};
use serde::{Deserialize, Serialize};

use crate::{
    middleware::{
        CallbackClosureRaw, Metadata, Middleware, MiddlewareRaw, PyMiddleware, PySerializer,
        Serializer, SerializerEnum,
    },
    types::TimeStamp,
};

#[pyclass]
pub struct ROS2Serializer;

impl Serializer for ROS2Serializer {
    fn serializer(&self) -> SerializerEnum {
        SerializerEnum::from(Self {})
    }

    fn serialize<T: Serialize>(&self, data: &T) -> Option<Vec<u8>> {
        serde_json::to_vec(data).ok()
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, payload: &[u8]) -> Option<T> {
        serde_json::from_slice::<T>(payload).ok()
    }
}

#[pyclass]
pub struct ROS2Middleware {
    node: Arc<Node>,
    publishers: Mutex<HashMap<String, Arc<Publisher>>>,
    subscribers: Mutex<HashMap<String, Arc<Subscriber>>>,
}

impl ROS2Middleware {
    pub fn new() -> Self {
        let node = r2r::Node::create("aerosim", "").unwrap();
        ROS2Middleware {
            node: Arc::new(node),
            publishers: Mutex::new(HashMap::new()),
            subscribers: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MiddlewareRaw for ROS2Middleware {
    async fn publish_raw(
        &self,
        message_type: &str,
        topic: &str,
        payload: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let publisher: Arc<Publisher> = {
            let mut publishers = self.publishers.lock().unwrap();
            
            if !publishers.contains_key(topic) {
                let publisher = self.node.create_publisher::<Vec<u8>>(topic)?;
                publishers.insert(topic.to_string(), Arc::new(publisher));
            }
            
            publishers.get(topic).unwrap().clone()
        };

        publisher.publish(payload.to_vec())?;
        Ok(())
    }

    async fn subscribe_raw(
        &self,
        message_type: &str,
        topic: &str,
        callback: CallbackClosureRaw,
    ) -> Result<(), Box<dyn Error>> {
        let subscriber = self.node.create_subscription::<Vec<u8>>(topic, move |msg| {
            if let Err(e) = callback(&msg) {
                eprintln!("Error in ROS2 callback: {}", e);
            }
        })?;

        {
            let mut subscribers = self.subscribers.lock().unwrap();
            subscribers.insert(topic.to_string(), Arc::new(subscriber));
        }

        Ok(())
    }

    async fn subscribe_all_raw(
        &self,
        topics: Vec<(String, String)>,
        callback: CallbackClosureRaw,
    ) -> Result<(), Box<dyn Error>> {
        for (message_type, topic) in topics {
            self.subscribe_raw(&message_type, &topic, callback.clone()).await?;
        }
        Ok(())
    }

    fn shutdown_raw(&self) {
        // Clear publishers and subscribers
        {
            let mut publishers = self.publishers.lock().unwrap();
            publishers.clear();
        }
        {
            let mut subscribers = self.subscribers.lock().unwrap();
            subscribers.clear();
        }
    }
}

#[async_trait]
impl Middleware for ROS2Middleware {
    fn get_serializer(&self) -> SerializerEnum {
        SerializerEnum::from(ROS2Serializer {})
    }
}

impl PyMiddleware for ROS2Middleware {}

#[pymethods]
impl ROS2Middleware {
    #[new]
    fn pynew(_py: Python) -> PyResult<Self> {
        Ok(Self::new())
    }

    #[pyo3(name = "publish")]
    #[pyo3(signature = (topic, message, timestamp_sim=None))]
    fn pypublish(
        &self,
        py: Python,
        topic: &str,
        message: PyObject,
        timestamp_sim: Option<TimeStamp>,
    ) -> PyResult<()> {
        futures::executor::block_on(self.pypublish_impl(py, topic, message, timestamp_sim))
    }

    #[pyo3(name = "subscribe")]
    fn pysubscribe(
        &self,
        py: Python,
        message_type: PyObject,
        topic: &str,
        callback: PyObject,
    ) -> PyResult<()> {
        futures::executor::block_on(self.pysubscribe_impl(py, message_type, topic, callback))
    }

    #[pyo3(name = "subscribe_all")]
    fn pysubscribe_all(
        &self,
        py: Python,
        message_type: PyObject,
        topics: Vec<String>,
        callback: PyObject,
    ) -> PyResult<()> {
        futures::executor::block_on(self.pysubscribe_all_impl(py, message_type, topics, callback))
    }

    #[pyo3(name = "publish_raw")]
    fn pypublish_raw(
        &self,
        py: Python,
        message_type: &str,
        topic: &str,
        payload: Py<pyo3::types::PyBytes>,
    ) -> PyResult<()> {
        futures::executor::block_on(self.pypublish_raw_impl(py, message_type, topic, payload))
    }

    #[pyo3(name = "subscribe_raw")]
    fn pysubscribe_raw(
        &self,
        py: Python<'_>,
        message_type: &str,
        topic: &str,
        callback: PyObject,
    ) -> PyResult<()> {
        futures::executor::block_on(self.pysubscribe_raw_impl(py, message_type, topic, callback))
    }

    #[pyo3(name = "subscribe_all_raw")]
    fn pysubscribe_all_raw(
        &self,
        py: Python,
        topics: Vec<(String, String)>,
        callback: PyObject,
    ) -> PyResult<()> {
        futures::executor::block_on(self.pysubscribe_all_raw_impl(py, topics, callback))
    }
}

impl PySerializer for ROS2Serializer {}

#[pymethods]
impl ROS2Serializer {
    #[new]
    fn pynew(_py: Python) -> PyResult<Self> {
        Ok(Self {})
    }

    #[pyo3(name = "serialize_message")]
    fn pyserialize_message(
        &self,
        py: Python<'_>,
        metadata: Metadata,
        data: PyObject,
    ) -> Option<Vec<u8>> {
        let serializer = SerializerEnum::from(ROS2Serializer {});
        self.pyserialize_message_impl(py, &serializer, metadata, data)
    }

    #[pyo3(name = "deserialize_message")]
    fn pydeserialize_message(
        &self,
        py: Python<'_>,
        message_type: PyObject,
        payload: &[u8],
    ) -> Option<(Metadata, PyObject)> {
        let serializer = SerializerEnum::from(ROS2Serializer {});
        self.pydeserialize_message_impl(py, &serializer, message_type, payload)
    }

    #[pyo3(name = "deserialize_metadata")]
    fn pydeserialize_metadata(&self, py: Python<'_>, payload: &[u8]) -> Option<Metadata> {
        let serializer = SerializerEnum::from(ROS2Serializer {});
        self.pydeserialize_metadata_impl(py, &serializer, payload)
    }

    #[pyo3(name = "deserialize_data")]
    fn pydeserialize_data(
        &self,
        py: Python<'_>,
        message_type: PyObject,
        payload: &[u8],
    ) -> Option<PyObject> {
        let serializer = SerializerEnum::from(ROS2Serializer {});
        self.pydeserialize_data_impl(py, &serializer, message_type, payload)
    }

    #[pyo3(name = "from_json")]
    fn pyserialize_from_json(
        &self,
        py: Python<'_>,
        type_name: &str,
        metadata: &Metadata,
        data: PyObject,
    ) -> Option<Vec<u8>> {
        let serializer = SerializerEnum::from(ROS2Serializer {});
        self.pyserialize_from_json_impl(py, &serializer, type_name, metadata, data)
    }

    #[pyo3(name = "to_json")]
    fn pydeserialize_to_json(
        &self,
        py: Python<'_>,
        type_name: &str,
        payload: &[u8],
    ) -> Option<PyObject> {
        let serializer = SerializerEnum::from(ROS2Serializer {});
        self.pydeserialize_to_json_impl(py, &serializer, type_name, payload)
    }
}
