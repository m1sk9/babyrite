use serenity::model::Timestamp;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Debug, Clone)]
pub struct CitationMessage {
    // メッセージ内容
    pub content: String,

    // 添付ファイル (画像/GIFのみ)
    pub attachment_image_url: Option<String>,

    // メッセージ送信者情報
    pub author: CitationMessageAuthor,

    // メッセージを送信したチャンネルの名前
    pub channel_name: String,

    // メッセージの送信日時
    pub create_at: Timestamp,
}

#[derive(TypedBuilder, Debug, Clone)]
pub struct CitationMessageAuthor {
    // メッセージ送信者のユーザーネーム
    pub name: String,

    // メッセージ送信者のアイコンURL
    pub icon_url: Option<String>,
}
