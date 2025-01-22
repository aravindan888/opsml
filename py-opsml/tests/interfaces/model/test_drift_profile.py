from opsml.scouter.drift import (
    PsiDriftConfig,
    CustomMetric,
    CustomMetricDriftConfig,
    CustomDriftProfile,
)
from typing import Tuple
from opsml.scouter.alert import AlertThreshold
from opsml.model import SklearnModel
from opsml.data import DataType
from pathlib import Path
import pandas as pd
from sklearn import ensemble  # type: ignore
from sklearn.preprocessing import StandardScaler  # type: ignore
from opsml.model import TaskType


def test_model_interface_drift_profile(
    tmp_path: Path,
    example_dataframe: Tuple[pd.DataFrame, pd.DataFrame, pd.DataFrame, pd.DataFrame],
):
    """
    Test drift profile creation for a model interface.

    1. Create profile and add to init
    2. create profile via create_drift_profile method for various types
    3. Test saving
    4. Test loading
    """

    X, y, _, _ = example_dataframe

    reg = ensemble.RandomForestClassifier(n_estimators=5)
    reg.fit(X.to_numpy(), y)

    # custom
    metric = CustomMetric(
        name="custom1", value=0.5, alert_threshold=AlertThreshold.Above
    )

    custom_profile1 = CustomDriftProfile(CustomMetricDriftConfig(), [metric])

    model = SklearnModel(
        model=reg,
        sample_data=X,
        task_type=TaskType.Classification,
        preprocessor=StandardScaler(),
        drift_profile=custom_profile1,
    )

    # create spc
    model.create_drift_profile(X)

    # create psi
    model.create_drift_profile(X, PsiDriftConfig(), DataType.Pandas)

    # custom
    metric = CustomMetric(
        name="custom", value=0.5, alert_threshold=AlertThreshold.Above
    )
    model.create_drift_profile([metric], CustomMetricDriftConfig())

    # save
    model.save(tmp_path)
