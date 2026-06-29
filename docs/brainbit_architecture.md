# BrainBit library — layered architecture

```mermaid
flowchart TB

    subgraph L1["Layer 1 — BLE transport (btleplug)"]
        direction LR
        T1["Scan / connect"]
        T2["Notify subscriptions"]
        T3["Write commands"]
    end

    subgraph L2["Layer 2 — Device protocol (pure decoders, no async)"]
        direction LR
        P1["EEG decoder<br/>4 channels, raw frames"]
        P2["Resistance decoder<br/>N packets per channel<br/>avg + status"]
        P3["Battery decoder<br/>charge level"]
    end

    subgraph L3["Layer 3 — Device core (state machine)"]
        direction TB
        subgraph L3A["--"]
            direction LR
            S1["ConnectionState<br/>connect / disconnect / reconnect"]
            S2["MeasurementMode<br/>EEG ↔ Resistance scan"]
            S3["ChannelQuality<br/>good / bad per channel"]
        end
        subgraph L3B["----"]
            direction LR
            SCH["ResistanceScheduler<br/>timer-driven, config-based"]
            REC["ReconnectPolicy<br/>backoff, retry attempts"]
        end
    end

    subgraph L4["Layer 4 — Public API"]
        direction LR
        E1["eeg_update"]
        E2["channel_quality_update"]
        E3["battery_update"]
        E4["connection_state_changed"]
    end

    subgraph L5["Layer 5 — Consumer side"]
        direction LR
        REC2["Recorder / Streamer<br/>video + audio frames,<br/>local recording"]
        APP["Consumer app<br/>visualization, UI,<br/>business logic"]
    end

    L1 --> L2
    L2 --> L3
    L3 --> L4
    L4 --> REC2
    L4 --> APP

    classDef transport fill:#E6F1FB,stroke:#185FA5,color:#042C53
    classDef protocol fill:#E6F1FB,stroke:#185FA5,color:#042C53
    classDef core fill:#E1F5EE,stroke:#0F6E56,color:#04342C
    classDef api fill:#FAECE7,stroke:#993C1D,color:#4A1B0C
    classDef consumer fill:#FBEAF0,stroke:#993556,color:#4B1528

    class T1,T2,T3 transport
    class P1,P2,P3 protocol
    class S1,S2,S3,SCH,REC core
    class E1,E2,E3,E4 api
    class REC2,APP consumer
```

## Layer responsibilities

- **BLE transport** — thin wrapper over `btleplug::Peripheral`: scanning, connect/disconnect, notify subscriptions, writing commands. No business logic; surfaces raw connection drop events.

- **Device protocol** — pure `&[u8] -> DomainType` functions, no `async`, no state. Easy to unit-test against fixtures without any BLE involved.

- **Device core** — the state machine: connection state, EEG ↔ resistance-scan mode switching (driven internally by a timer/config), per-channel quality tracking, reconnect policy.

- **Public API** — the `EventHandler` trait (via `async_trait`) is the single point of outward-facing events: EEG data, channel quality, battery level, connection state changes.

- **Consumer side** — anything implementing `EventHandler`: a `Recorder`/`Streamer` pushing frames to disk or RTMP, or the actual application with visualization and UI logic.
