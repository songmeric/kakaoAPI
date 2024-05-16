use std::collections::HashMap;

use futures::{channel::mpsc::channel, pin_mut};

use kiwi_talk_app::{
    app::{client::create_client_2, AppCredential},
    system::{DeviceInfo, DeviceUuid, SystemInfo},
};
use kiwi_talk_client::{
    channel::{ChannelDataVariant, ClientChannel},
    chat::{Chat, ChatContent, ChatType},
    status::ClientStatus,
    KiwiTalkClient,
};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use talk_api_client::{
    agent::TalkApiAgent,
    auth::{
        xvc::default::Win32XVCHasher, AccountLoginForm, AuthClientConfig, AuthDeviceConfig,
        LoginMethod, TalkAuthClient,
    },
};
use talk_loco_client::client::talk::TalkClient;
use talk_loco_command::request::chat::{ChatInfoReq, ChatOnRoomReq, DeleteMsgReq, HideMsgReq};

mod kakao;

pub const CONFIG: AuthClientConfig = AuthClientConfig {
    device: AuthDeviceConfig {
        // Device name
        name: "TEST_DEVICE",

        model: None,
        // Unique id base64 encoded. 62 bytes
        // uuid: "",
        uuid: ""
    },
    // lang
    language: "ko",
    // Talk client version
    version: "3.4.7",
    // Talk agent
    agent: TalkApiAgent::Win32("10.0"),
};

// XVC hasher
pub const HASHER: Win32XVCHasher = Win32XVCHasher("JAYDEN", "JAYMOND");

#[tokio::main]
async fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("init logger");

    let auth_client = TalkAuthClient::new(CONFIG, HASHER);

    let login_form = LoginMethod::Account(AccountLoginForm {
        // email: "scheitlinhirokoie401@gmail.com",
        // email: "esmaraldaaleccao91626@gmail.com",
        // email: "claudinelinneml508@gmail.com",
        // email: "lizakfrancesca877167@gmail.com",
        email: "",
        // password: "",
        password: "",
    });

    let res = auth_client
        .login(
            login_form, // Force login
            true,
        )
        .await
        .expect("log in");

    println!("res result: {:#?}", res);

    let res_data = res.data.expect("res data");
    let cred = res_data.credential;
    let credential = AppCredential {
        access_token: cred.access_token,
        refresh_token: cred.refresh_token,
        user_id: Some(res_data.user_id as i64),
    };

    let client_status = ClientStatus::Unlocked;
    let system_info = SystemInfo {
        device_data_dir: "device_data_dir".into(),
        data_dir: "data_dir".into(),
        device_info: DeviceInfo {
            locale: "KR".into(),
            name: CONFIG.device.name.into(),
            device_uuid: DeviceUuid(CONFIG.device.uuid.into()),
        },
    };

    let (sender, mut recv) = channel(256);
    let (client, channels): (KiwiTalkClient, HashMap<i64, ChannelDataVariant>) =
        create_client_2(&credential, client_status, &system_info, sender)
            .await
            .expect("create client");

    println!("channels: {:#?}", channels);

    let channel_id = *channels.keys().nth(0).unwrap();
    println!("channel id: {} {:?}", channel_id, channels[&channel_id]);

    // Hide:
    // Msg: Some(Chat(Chat(ChatReceived { channel_id: 18384376529312404, link_id: Some(283272051), log_id: 3031656572561932288, user_nickname: Some("Eaves"), chat: Chatlog { log_id: 3031656572561932288, prev_log_id: Some(3031645847147776001), channel_id: 18384376529312404, sender_id: 5584384776934255213, send_at: 1682338826, chat: Chat { chat_type: ChatType(1), content: ChatContent { message: Some("Try to hide this"), attachment: Some("{}"), supplement: None }, message_id: 859208614 }, referer: None } })))

    let mut done = true;

    loop {
        while let Some(msg) = recv.try_next().ok() {
            println!("Msg: {:?}", msg);

            // Important requests:
            // [x] Send
            // [x] Delete
            // [ ] Hide
            // [ ] Kick

            // Extra ones:
            // [ ] Join chat

            if !done {
                done = true;

                // let channel_id = 370092996731938;
                // let res = ClientChannel::new(channel_id, &client.connection())
                //     .send_chat(
                //         Chat {
                //             chat_type: ChatType::TEXT,
                //             content: ChatContent {
                //                 message: Some("Hello".into()),
                //                 attachment: None,
                //                 supplement: None,
                //             },
                //             message_id: 0,
                //         },
                //         true,
                //     )
                //     .await
                //     .expect("send chat res");
                // println!("Res: {:?}", res);

                let client = TalkClient(&client.connection().session);
                let res = client
                    .delete_chat(&DeleteMsgReq {
                        chat_id: 370092996731938,
                        log_id: 3032236880424882176,
                    })
                    .await
                    .expect("delete msg");
                println!("Res: {:?}", res);

                // let client = TalkClient(&client.connection().session);
                // let res = client
                //     .hide_chat(&HideMsgReq {
                //         link_id: 283272051,
                //         channel_id: 18384376529312404,
                //         log_id: 3031656572561932288,
                //         chat_type: 1,
                //     })
                //     .await
                //     .expect("hide msg");
                // println!("Res: {:?}", res);
            }
        }
    }

    // let chat_id = *channels.keys().nth(0).unwrap();
    // println!("chat id: {}", chat_id);

    // let client = TalkClient(&client.connection().session);
    // let chan_info = client
    //     .channel_info(&ChatInfoReq { chat_id })
    //     .await
    //     .expect("chat info");

    // println!("Chan info: {:#?}", chan_info);

    // let client = TalkClient(&client.connection().session);
    // let stream = client.channel_list_stream(0, None);
    // pin_mut!(stream);

    // while let Some(res) = stream.try_next().await.unwrap() {
    //     println!("Recv: {:?}", res);
    // }

    // let channels: HashMap<i64, ChannelDataVariant> =
    //     client.load_channel_list().await.expect("channels");
    // println!("Channels: {:#?}", channels);

    // let talk_client = TalkClient(client.connection().session);
}
