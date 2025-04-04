import type {
  BinnedCustomMetrics,
  BinnedPsiFeatureMetrics,
  BinnedSpcFeatureMetrics,
} from "./types";
import type { Alert } from "$lib/components/monitoring/alert/types";

const sampleCustomMetrics: BinnedCustomMetrics = {
  metrics: {
    custom: {
      metric: "custom",
      created_at: [
        "2025-03-25 00:43:59",
        "2025-03-26 10:00:00",
        "2025-03-27 11:00:00",
        "2025-03-28 12:00:00",
        "2025-03-29 12:00:00",
      ],
      stats: [
        {
          avg: 0.95,
          lower_bound: 0.92,
          upper_bound: 0.98,
        },
        {
          avg: 0.94,
          lower_bound: 0.91,
          upper_bound: 0.97,
        },
        {
          avg: 0.96,
          lower_bound: 0.93,
          upper_bound: 0.99,
        },
        {
          avg: 0.9,
          lower_bound: 0.93,
          upper_bound: 0.99,
        },
        {
          avg: 0.4,
          lower_bound: 0.93,
          upper_bound: 0.99,
        },
      ],
    },
    f1_score: {
      metric: "f1_score",
      created_at: [
        "2024-03-26T10:00:00",
        "2024-03-26T11:00:00",
        "2024-03-26T12:00:00",
      ],
      stats: [
        {
          avg: 0.88,
          lower_bound: 0.85,
          upper_bound: 0.91,
        },
        {
          avg: 0.87,
          lower_bound: 0.84,
          upper_bound: 0.9,
        },
        {
          avg: 0.89,
          lower_bound: 0.86,
          upper_bound: 0.92,
        },
      ],
    },
    latency_ms: {
      metric: "latency_ms",
      created_at: [
        "2024-03-26T10:00:00",
        "2024-03-26T11:00:00",
        "2024-03-26T12:00:00",
      ],
      stats: [
        {
          avg: 150.5,
          lower_bound: 145.0,
          upper_bound: 156.0,
        },
        {
          avg: 148.2,
          lower_bound: 143.5,
          upper_bound: 153.0,
        },
        {
          avg: 152.8,
          lower_bound: 147.2,
          upper_bound: 158.4,
        },
      ],
    },
  },
};

const sampleSpcMetrics: BinnedSpcFeatureMetrics = {
  features: {
    col_0: {
      created_at: [
        "2025-03-25 00:43:59",
        "2025-03-26 10:00:00",
        "2025-03-27 11:00:00",
        "2025-03-28 12:00:00",
        "2025-03-29 12:00:00",
      ],
      values: [100, 105, 200, 1025, 101],
    },
    col_1: {
      created_at: [
        "2025-03-25 00:43:59",
        "2025-03-26 10:00:00",
        "2025-03-27 11:00:00",
        "2025-03-28 12:00:00",
        "2025-03-29 12:00:00",
      ],
      values: [100, 105, 200, 1025, 101],
    },
  },
};

const samplePsiMetrics: BinnedPsiFeatureMetrics = {
  features: {
    col_0: {
      created_at: [
        "2025-03-25 00:43:59",
        "2025-03-26 10:00:00",
        "2025-03-27 11:00:00",
        "2025-03-28 12:00:00",
        "2025-03-29 12:00:00",
      ],
      psi: [0.05, 0.07, 0.04, 0.1, 0.05],
      overall_psi: 0.053,
      bins: {
        0: 0.1,
        1: 0.2,
        2: 0.3,
        3: 0.25,
        4: 0.15,
      },
    },
    col_1: {
      created_at: [
        "2025-03-25 00:43:59",
        "2025-03-25 01:43:59",
        "2025-03-25 02:43:59",
        "2025-03-25 03:43:59",
        "2025-03-25 04:43:59",
        "2025-03-25 05:43:59",
        "2025-03-25 06:43:59",
        "2025-03-25 07:43:59",
        "2025-03-25 08:43:59",
        "2025-03-25 09:43:59",
      ],
      psi: [1, 2, 3, 4, 5, 1, 2, 3, 4, 5],
      overall_psi: 0.053,
      bins: {
        0: 0.1,
        1: 0.2,
        2: 0.3,
        3: 0.25,
        4: 0.15,
      },
    },
  },
};

export { sampleSpcMetrics, samplePsiMetrics, sampleCustomMetrics };

export const sampleAlerts: Alert[] = [
  {
    created_at: "2024-03-28 10:30:00",
    name: "credit_model",
    space: "models",
    version: "1.0.0",
    feature: "income",
    alert: { type: "drift_detected", message: "PSI value exceeded threshold" },
    id: 1,
    status: "active",
  },
  {
    created_at: "2024-03-28 09:45:00",
    name: "fraud_detection",
    space: "models",
    version: "2.1.0",
    feature: "transaction_amount",
    alert: { type: "spc_violation", message: "Value outside control limits" },
    id: 2,
    status: "resolved",
  },
  {
    created_at: "2024-03-28 09:00:00",
    name: "customer_churn",
    space: "ml_models",
    version: "1.2.3",
    feature: "usage_frequency",
    alert: { type: "custom_metric", message: "Metric below threshold" },
    id: 3,
    status: "active",
  },
  {
    created_at: "2024-03-27 23:15:00",
    name: "recommendation_engine",
    space: "recsys",
    version: "3.0.1",
    feature: "user_engagement",
    alert: { type: "drift_detected", message: "Distribution shift detected" },
    id: 4,
    status: "pending",
  },
  {
    created_at: "2024-03-27 22:30:00",
    name: "credit_model",
    space: "models",
    version: "1.0.0",
    feature: "debt_ratio",
    alert: { type: "spc_violation", message: "Consecutive points above mean" },
    id: 5,
    status: "active",
  },
  {
    created_at: "2024-03-27 21:45:00",
    name: "fraud_detection",
    space: "models",
    version: "2.1.0",
    feature: "ip_velocity",
    alert: { type: "psi_threshold", message: "PSI above 0.2" },
    id: 6,
    status: "investigating",
  },
  {
    created_at: "2024-03-27 20:00:00",
    name: "price_optimization",
    space: "pricing",
    version: "1.1.0",
    feature: "demand_forecast",
    alert: { type: "custom_metric", message: "Accuracy below target" },
    id: 7,
    status: "resolved",
  },
  {
    created_at: "2024-03-27 19:15:00",
    name: "customer_churn",
    space: "ml_models",
    version: "1.2.3",
    feature: "support_tickets",
    alert: { type: "drift_detected", message: "Significant feature drift" },
    id: 8,
    status: "active",
  },
  {
    created_at: "2024-03-27 18:30:00",
    name: "recommendation_engine",
    space: "recsys",
    version: "3.0.1",
    feature: "click_through_rate",
    alert: { type: "spc_violation", message: "Point beyond 3 sigma" },
    id: 9,
    status: "resolved",
  },
  {
    created_at: "2024-03-27 17:45:00",
    name: "price_optimization",
    space: "pricing",
    version: "1.1.0",
    feature: "competitor_prices",
    alert: { type: "custom_metric", message: "Data freshness warning" },
    id: 10,
    status: "active",
  },
];
