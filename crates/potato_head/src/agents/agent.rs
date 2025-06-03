use crate::agents::provider::openai::OpenAIClient;
use crate::agents::provider::types::Provider;
use crate::Message;
use crate::{
    agents::client::GenAiClient, agents::task::Task, agents::types::AgentResponse,
    error::AgentError,
};
use opsml_state::app_state;
use opsml_utils::create_uuid7;
use pyo3::prelude::*;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone)]
#[pyclass]
pub struct Agent {
    #[pyo3(get)]
    pub id: String,

    client: GenAiClient,
}

#[pymethods]
impl Agent {
    #[new]
    #[pyo3(signature = (provider))]
    /// Creates a new Agent instance.
    ///
    /// # Arguments:
    /// * `provider` - A Python object representing the provider, expected to be an a variant of Provider or a string
    /// that can be mapped to a provider variant
    ///
    pub fn new(provider: &Bound<'_, PyAny>) -> Result<Self, AgentError> {
        let provider = Provider::extract_provider(provider)?;

        let client = match provider {
            Provider::OpenAI => GenAiClient::OpenAI(OpenAIClient::new(None, None, None)?),
            // Add other providers here as needed
        };

        Ok(Self {
            client,
            id: create_uuid7(),
        })
    }

    #[pyo3(signature = (task, context_messages = HashMap::new()))]
    pub fn execute_task(
        &self,
        task: &Task,
        context_messages: HashMap<String, Vec<Message>>,
    ) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task");
        // we need to clone in order to not modify the original task
        let mut cloned_task = task.clone();

        if !cloned_task.dependencies.is_empty() {
            for dep in &cloned_task.dependencies {
                if let Some(messages) = context_messages.get(dep) {
                    for message in messages {
                        // prepend the messages from dependencies
                        cloned_task.prompt.user_message.insert(0, message.clone());
                    }
                }
            }
        }

        let chat_response = app_state()
            .runtime
            .block_on(async { self.client.execute(&cloned_task.prompt).await })?;

        Ok(AgentResponse::new(task.id.clone(), chat_response))
    }
}

/// Rust method implementation of the Agent
impl Agent {
    pub async fn execute_async_task(
        &self,
        task: &Task,
        context_messages: HashMap<String, Vec<Message>>,
    ) -> Result<AgentResponse, AgentError> {
        // Extract the prompt from the task
        debug!("Executing task: {}", task.id);
        let mut cloned_task = task.clone();

        if !cloned_task.dependencies.is_empty() {
            for dep in &cloned_task.dependencies {
                if let Some(messages) = context_messages.get(dep) {
                    for message in messages {
                        // prepend the messages from dependencies
                        cloned_task.prompt.user_message.insert(0, message.clone());
                    }
                }
            }
        }

        // Use the client to execute the task
        let chat_response = self.client.execute(&cloned_task.prompt).await?;

        Ok(AgentResponse::new(task.id.clone(), chat_response))
    }
}
