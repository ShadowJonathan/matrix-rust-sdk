use std::env;

use matrix_sdk::{
    api_appservice,
    api_appservice::Registration,
    async_trait,
    events::{room::member::MemberEventContent, AnyRoomEvent, AnyStateEvent, SyncStateEvent},
    room::Room,
    EventHandler, Raw,
};
use matrix_sdk_appservice::*;
use matrix_sdk_test::async_test;
use serde_json::json;

fn registration_string() -> String {
    include_str!("../tests/registration.yaml").to_owned()
}

async fn appservice(registration: Option<Registration>) -> Result<Appservice> {
    env::set_var("RUST_LOG", "mockito=debug,matrix_sdk=debug");
    let _ = tracing_subscriber::fmt::try_init();

    let registration = match registration {
        Some(registration) => registration.into(),
        None => AppserviceRegistration::try_from_yaml_str(registration_string()).unwrap(),
    };

    let homeserver_url = mockito::server_url();
    let server_name = "localhost";

    Ok(Appservice::new(homeserver_url.as_ref(), server_name, registration).await?)
}

fn member_json() -> serde_json::Value {
    json!({
        "content": {
            "avatar_url": null,
            "displayname": "example",
            "membership": "join"
        },
        "event_id": "$151800140517rfvjc:localhost",
        "membership": "join",
        "origin_server_ts": 151800140,
        "room_id": "!ahpSDaDUPCCqktjUEF:localhost",
        "sender": "@example:localhost",
        "state_key": "@example:localhost",
        "type": "m.room.member",
        "prev_content": {
            "avatar_url": null,
            "displayname": "example",
            "membership": "invite"
        },
        "unsigned": {
            "age": 297036,
            "replaces_state": "$151800111315tsynI:localhost"
        }
    })
}

#[async_test]
async fn test_event_handler() -> Result<()> {
    let mut appservice = appservice(None).await?;

    struct Example {}

    impl Example {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait]
    impl EventHandler for Example {
        async fn on_state_member(&self, room: Room, event: &SyncStateEvent<MemberEventContent>) {
            dbg!(room, event);
        }
    }

    appservice.set_event_handler(Box::new(Example::new())).await?;

    let event = serde_json::from_value::<AnyStateEvent>(member_json()).unwrap();
    let event: Raw<AnyRoomEvent> = AnyRoomEvent::State(event).into();
    let events = vec![event];

    let incoming = api_appservice::event::push_events::v1::IncomingRequest::new(
        "any_txn_id".to_owned(),
        events,
    );

    appservice.get_cached_client(None)?.receive_transaction(incoming).await?;

    Ok(())
}

#[async_test]
async fn test_transaction() -> Result<()> {
    let appservice = appservice(None).await?;

    let event = serde_json::from_value::<AnyStateEvent>(member_json()).unwrap();
    let event: Raw<AnyRoomEvent> = AnyRoomEvent::State(event).into();
    let events = vec![event];

    let incoming = api_appservice::event::push_events::v1::IncomingRequest::new(
        "any_txn_id".to_owned(),
        events,
    );

    appservice.get_cached_client(None)?.receive_transaction(incoming).await?;

    Ok(())
}

#[async_test]
async fn test_verify_hs_token() -> Result<()> {
    let appservice = appservice(None).await?;

    let registration = appservice.registration();

    assert!(appservice.compare_hs_token(&registration.hs_token));

    Ok(())
}

mod registration {
    use super::*;

    #[test]
    fn test_registration() -> Result<()> {
        let registration: Registration = serde_yaml::from_str(&registration_string())?;
        let registration: AppserviceRegistration = registration.into();

        assert_eq!(registration.id, "appservice");

        Ok(())
    }

    #[test]
    fn test_registration_from_yaml_file() -> Result<()> {
        let registration = AppserviceRegistration::try_from_yaml_file("./tests/registration.yaml")?;

        assert_eq!(registration.id, "appservice");

        Ok(())
    }

    #[test]
    fn test_registration_from_yaml_str() -> Result<()> {
        let registration = AppserviceRegistration::try_from_yaml_str(registration_string())?;

        assert_eq!(registration.id, "appservice");

        Ok(())
    }
}
