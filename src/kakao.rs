use std::collections::{hash_map::Entry, HashMap};

use anyhow::{Context, Result};
use futures::{
    channel::mpsc::{channel, Receiver},
    StreamExt,
};
use kiwi_talk_app::{
    app::{client::create_client_2, AppCredential},
    system::{DeviceInfo, DeviceUuid, SystemInfo},
};
use kiwi_talk_client::{
    channel::{ChannelDataVariant, ClientChannel},
    chat::{Chat, Chatlog},
    error::KiwiTalkClientError,
    event::{chat::ChatEvent, KiwiTalkClientEvent},
    status::ClientStatus,
    KiwiTalkClient,
};
use log::*;
use talk_api_client::{
    agent::TalkApiAgent,
    auth::{
        xvc::default::Win32XVCHasher, AccountLoginForm, AuthClientConfig, AuthDeviceConfig,
        LoginMethod, TalkAuthClient,
    },
};
use talk_loco_client::client::{talk::TalkClient, ClientRequestError};
use talk_loco_command::{
    request::chat::{
        join_channel::JoinChannelReqProfile, CheckJoinReq, DeleteMsgReq, GetChatLogsReq,
        HideMsgReq, JoinChannelReq, JoinInfoReq, KickUserReq,
    },
    response::chat::{join_channel::ChatRoomMember, JoinChannelRes},
    structs::{chat::Chatlog as Chatlog2, openlink::OpenLinkUser, user::DisplayUserInfo},
};

pub struct KakaoClientCfg<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

pub struct KakaoClient {
    pub talk_client: KiwiTalkClient,
    pub talk_event_recv: Receiver<KiwiTalkClientEvent>,
    pub initial_channels: HashMap<i64, ChannelDataVariant>,
    pub known_users: HashMap<i64, KakaoUser>,
}

impl KakaoClient {
    pub async fn new(cfg: KakaoClientCfg<'_>) -> Result<Self> {
        info!("New Kakao client");

        let config = AuthClientConfig {
            device: AuthDeviceConfig {
                name: "TEST_DEVICE",
                model: None,
                uuid: "6SMj9g0xFYodk+ItC4FKX1EgnZiLPibCWGXuqIgEwx56uwoFJg+GCX0YneaczW4yEt3QzbI+6Hhz9env9cV6wQ=="
            },
            language: "ko",
            version: "3.4.7",
            agent: TalkApiAgent::Win32("10.0"),
        };
        let hasher = Win32XVCHasher("JAYDEN", "JAYMOND");

        info!("Logging in...");
        let auth_client = TalkAuthClient::new(config, hasher);
        let login_form = LoginMethod::Account(AccountLoginForm {
            email: cfg.email,
            password: cfg.password,
        });
        let login_response = auth_client.login(login_form, true).await.context("login")?;
        let login_data = login_response.data.context("login response data")?;
        info!("Logged in");

        // NOTE: This part below is from the Kiwi App
        info!("Starting Kiwi app client...");
        let credential = AppCredential {
            access_token: login_data.credential.access_token,
            refresh_token: login_data.credential.refresh_token,
            user_id: Some(login_data.user_id as i64),
        };
        let client_status = ClientStatus::Unlocked;
        let system_info = SystemInfo {
            device_data_dir: "device_data_dir".into(),
            data_dir: "data_dir".into(),
            device_info: DeviceInfo {
                locale: "KR".into(),
                name: config.device.name.into(),
                device_uuid: DeviceUuid(config.device.uuid.into()),
            },
        };

        let (sender, recv) = channel(256);
        let (client, channels): (KiwiTalkClient, HashMap<i64, ChannelDataVariant>) =
            create_client_2(&credential, client_status, &system_info, sender)
                .await
                .context("create client")?;
        info!("Started Kiwi app client");

        Ok(Self {
            talk_client: client,
            talk_event_recv: recv,
            initial_channels: channels,
            known_users: HashMap::new(),
        })
    }

    pub async fn next_event(&mut self) -> Result<KiwiTalkClientEvent> {
        let msg = self.talk_event_recv.next().await.context("kiwi event")?;
        info!("Received message: {:?}", msg);

        match &msg {
            KiwiTalkClientEvent::Chat(ChatEvent::Chat(e)) => {
                if let Some(nickname) = e.user_nickname.clone() {
                    match self.known_users.entry(e.chat.sender_id) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().nickname = nickname;
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(KakaoUser {
                                user_id: e.chat.sender_id,
                                nickname,
                                image_url: None,
                            });
                        }
                    }
                }
            }
            KiwiTalkClientEvent::ProfileChanged(e) => {
                self.known_users
                    .insert(e.open_link_user.user_id, e.open_link_user.clone().into());
            }
            KiwiTalkClientEvent::Unhandled(e) => warn!("Unhandled event: {:?}", e),
            KiwiTalkClientEvent::Error(err) => error!("Error event: {:?}", err),
            _ => (),
        }

        Ok(msg)
    }

    pub async fn join_channel(
        &mut self,
        link_url: &str,
        nickname: &str,
        profile_path: Option<&str>,
        passcode: Option<&str>,
    ) -> Result<JoinChannelRes, KiwiTalkClientError> {
        info!(
            "Join channel via link '{}' with passcode: {:?}",
            link_url,
            passcode.is_some()
        );

        let client = TalkClient(&self.talk_client.connection().session);

        info!("Get join info");
        let join_info = client
            .get_join_info(&JoinInfoReq {
                link_url: link_url.into(),
                referer: "EW".to_owned(),
            })
            .await?;
        info!("Join info: {:?}", join_info);

        let token = match passcode {
            Some(passcode) => {
                info!("Check join");
                let check_join = client
                    .check_join(&CheckJoinReq {
                        link_id: join_info.open_link.link_id,
                        passcode: passcode.to_owned(),
                    })
                    .await?;
                info!("Check join: {:?}", check_join);
                Some(check_join.token)
            }
            None => None,
        };

        info!("Join channel");
        let join_channel_response = client
            .join_channel(&JoinChannelReq {
                link_id: join_info.open_link.link_id,
                referer: "EW:".to_owned(),
                profile: JoinChannelReqProfile::KakaoAnon {
                    ptp: 2,
                    nickname: nickname.to_owned(),
                    profile_path: profile_path.map(|x| x.to_owned()), // TODO: Does this do stuff?
                },
                token,
            })
            .await?;
        info!("Joined successfully");

        for user in join_channel_response.chat_room.members.iter() {
            self.known_users.insert(user.user_id, user.clone().into());
        }

        Ok(join_channel_response)
    }

    pub fn get_initial_channels(&self) -> &HashMap<i64, ChannelDataVariant> {
        &self.initial_channels
    }

    pub fn get_known_user_info(&self, user_id: i64) -> Option<&KakaoUser> {
        self.known_users.get(&user_id)
    }

    pub async fn get_chat_logs(
        &self,
        chat_id: i64,
        since: i64,
    ) -> Result<Vec<Chatlog2>, ClientRequestError> {
        info!("Get chat logs for chat_id={} since={}", chat_id, since);
        let client = TalkClient(&self.talk_client.connection().session);
        let res = client
            .get_chat_logs(&GetChatLogsReq {
                chat_ids: vec![chat_id],
                sinces: vec![since],
            })
            .await?;
        info!("Got chat logs successfully");
        Ok(res.chat_logs)
    }

    pub async fn send_message(
        &self,
        channel_id: i64,
        chat: Chat,
        no_seen: bool,
    ) -> Result<Chatlog, KiwiTalkClientError> {
        info!("Send chat to channel_id={} chat={:?}", channel_id, chat);
        let res = ClientChannel::new(channel_id, &self.talk_client.connection())
            .send_chat(chat, no_seen)
            .await?;
        info!("Sent chat successfully");
        Ok(res)
    }

    pub async fn delete_message(&self, req: DeleteMsgReq) -> Result<(), ClientRequestError> {
        info!("Delete message {:?}", req);
        let client = TalkClient(&self.talk_client.connection().session);
        let _res = client.delete_chat(&req).await?;
        info!("Deleted message successfully");
        Ok(())
    }

    pub async fn hide_message(&self, req: HideMsgReq) -> Result<(), ClientRequestError> {
        info!("Hide message {:?}", req);
        let client = TalkClient(&self.talk_client.connection().session);
        let _res = client.hide_chat(&req).await?;
        info!("Hid message successfully");
        Ok(())
    }

    pub async fn kick_user(&self, req: KickUserReq) -> Result<(), ClientRequestError> {
        info!("Kick user {:?}", req);
        let client = TalkClient(&self.talk_client.connection().session);
        let _res = client.kick_user(&req).await?;
        info!("Kicked user successfully");
        Ok(())
    }
}

pub struct KakaoUser {
    pub user_id: i64,
    pub nickname: String,
    pub image_url: Option<String>,
}

impl From<OpenLinkUser> for KakaoUser {
    fn from(value: OpenLinkUser) -> Self {
        Self {
            user_id: value.user_id,
            nickname: value.nickname,
            image_url: value.profile_image_url,
        }
    }
}

impl From<DisplayUserInfo> for KakaoUser {
    fn from(value: DisplayUserInfo) -> Self {
        Self {
            user_id: value.user_id,
            nickname: value.nickname,
            image_url: value.profile_image_url,
        }
    }
}

impl From<ChatRoomMember> for KakaoUser {
    fn from(value: ChatRoomMember) -> Self {
        Self {
            user_id: value.user_id,
            nickname: value.nickname,
            image_url: value.profile_image_url,
        }
    }
}
