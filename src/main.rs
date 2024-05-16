use anyhow::Result;
use kakao::{KakaoClient, KakaoClientCfg};
use kiwi_talk_client::chat::{Chat, ChatContent, ChatType};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use talk_loco_command::request::chat::{DeleteMsgReq, HideMsgReq};

mod kakao;

#[tokio::main]
async fn main() -> Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    let cfg = KakaoClientCfg {
        email: "#",
        password: "#",
    };

    let mut client = KakaoClient::new(cfg).await?;

    let chat_logs = client
        .get_chat_logs(18384565413113921, 3032501404205867008)
        .await?;

    println!("{:?}", chat_logs);

    // let channel = ChannelWrapper {
    //     link_id: 283608594,
    //     channel_id: 18384565413113921,
    // };

    let mut done = false;

    loop {
        let event = client.next_event().await?;

        println!("Got an event: {:?}", event);

        if !done {
            done = true;

            // channel
            //     .hide_message(&client, 3032496737807724544, 1)
            //     .await?;

            // client
            //     .delete_message(DeleteMsgReq {
            //         chat_id: 0,
            //         log_id: 0,
            //     })
            //     .await?;

            // client
            //     .send_message(
            //         18384565413113921,
            //         Chat {
            //             chat_type: ChatType::TEXT,
            //             content: ChatContent {
            //                 message: Some("Test".to_owned()),
            //                 attachment: None,
            //                 supplement: None,
            //             },
            //             message_id: 0,
            //         },
            //     )
            //     .await?;

            // client
            //     .hide_message(HideMsgReq {
            //         // link_id and channel_id are specific to the channel; log_id is the mssage id
            //         link_id: 283608594,
            //         channel_id: 18384565413113921,
            //         log_id: 3032496737807724544,
            //         chat_type: 1,
            //     })
            //     .await?;

            // client
            //     .kick_user(KickUserReq {
            //         channel_id: 18384565413113921,
            //         user_id: 6766397537925605521,
            //         link_id: 283608594,
            //     })
            //     .await?;

            // client
            //     .join_channel(
            //         "https://open.kakao.com/o/gfvKeahf",
            //         "codementor",
            //         None,
            //         Some("333111"),
            //     )
            //     .await?;
        }
    }
}

struct ChannelWrapper {
    link_id: i64,
    channel_id: i64,
}

impl ChannelWrapper {
    pub async fn hide_message(
        &self,
        client: &KakaoClient,
        log_id: i64,
        chat_type: i32,
    ) -> Result<()> {
        client
            .hide_message(HideMsgReq {
                link_id: self.link_id,
                channel_id: self.channel_id,
                log_id,
                chat_type,
            })
            .await?;
        Ok(())
    }
}
