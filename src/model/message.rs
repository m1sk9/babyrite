use serenity::model::Timestamp;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct CitationMessage {
    // メッセージ内容
    pub content: String,

    // メッセージ送信者のユーザーネーム
    pub author_name: String,

    // メッセージ送信者のアイコンURL
    pub author_avatar_url: Option<String>,

    // メッセージを送信したチャンネルの名前
    pub channel_name: String,

    // メッセージの送信日時
    pub create_at: Timestamp,
}
