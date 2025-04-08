use opsml_cards::{
    DataCard, DataCardMetadata, ExperimentCard, ModelCard, ModelCardMetadata, PromptCard,
    UidMetadata,
};
use opsml_deck::{Card, CardDeck, CardKwargs};
use opsml_registry::{CardRegistries, CardRegistry};
use opsml_types::contracts::{CardList, CardRecord};
use opsml_types::{cards::ComputeEnvironment, RegistryMode, RegistryType};

#[cfg(feature = "server")]
use opsml_registry::RegistryTestHelper;

use pyo3::prelude::*;

#[pymodule]
pub fn card(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CardRecord>()?;
    m.add_class::<CardList>()?;
    m.add_class::<DataCard>()?;
    m.add_class::<DataCardMetadata>()?;

    // opsml_registry
    m.add_class::<CardRegistry>()?;
    m.add_class::<CardRegistries>()?;
    m.add_class::<RegistryType>()?;
    m.add_class::<RegistryMode>()?;

    #[cfg(feature = "server")]
    m.add_class::<RegistryTestHelper>()?;

    // ModelCard
    m.add_class::<ModelCard>()?;
    m.add_class::<ModelCardMetadata>()?;

    // experimentcard
    m.add_class::<ExperimentCard>()?;
    m.add_class::<ComputeEnvironment>()?;
    m.add_class::<UidMetadata>()?;

    // promptcard
    m.add_class::<PromptCard>()?;

    // CardDeck
    m.add_class::<CardDeck>()?;
    m.add_class::<CardKwargs>()?;
    m.add_class::<Card>()?;

    Ok(())
}
