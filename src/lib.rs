use std::error::Error;

use mailparse::{addrparse, parse_mail, MailAddr, MailHeaderMap};

pub struct MyMailbox<'a> {
    host: &'a str,
    port: u16,
    user: &'a str,
    password: &'a str,
    selection: &'a str,
}
impl<'a> Default for MyMailbox<'a> {
    fn default() -> Self {
        Self {
            host: "",
            port: 993,
            user: "",
            password: "",
            selection: "INBOX",
        }
    }
}
#[derive(Debug)]
pub struct MyMessage {
    from: String,
    subject: String,
    body: String,
}

pub fn read_mail(mailbox: &MyMailbox) -> Result<Vec<MyMessage>, Box<dyn Error>> {
    let tls = native_tls::TlsConnector::builder().build()?;
    let client = imap::connect((mailbox.host, mailbox.port), mailbox.host, &tls)?;

    // ログイン
    let mut imap_session = client
        .login(mailbox.user, mailbox.password)
        .map_err(|e| e.0)?;

    // メールボックスを選択
    imap_session.select(mailbox.selection)?;

    // 全 uid を取得
    let uids = imap_session.uid_search("ALL")?;

    // 各 uid から MyMessage（from, subject, body）を抽出
    let messages = uids
        .iter()
        .map(|uid| {
            //（"RFC822"ではなく）"BODY.PEEK[]" を使うことにより既読にしない
            let messages = imap_session
                .uid_fetch(uid.to_string(), "BODY.PEEK[]")
                .unwrap();
            let message = messages.iter().next().unwrap();
            parse(message.body().unwrap()).unwrap()
        })
        .collect::<Vec<MyMessage>>();

    // ログアウト
    imap_session.logout()?;

    Ok(messages)
}

fn parse(raw_data: &[u8]) -> Result<MyMessage, Box<dyn Error>> {
    let parsed_mail = parse_mail(raw_data)?;
    let headers = &parsed_mail.headers;

    // 差出アドレス（メールアドレスのみ）
    let from = match &addrparse(&headers.get_first_value("From").ok_or("no From header(1)")?)?
        .first()
        .ok_or("no From header(2)")?
    {
        MailAddr::Single(info) => info.addr.to_string(),
        _ => return Err("no From header(3)".into()),
    };

    // 件名
    let subject = headers
        .get_first_value("Subject")
        .ok_or("no Subject header")?;

    // 本文
    // subparts がある場合は、最初の「mimetype: "text/plain"」になっているパートを使う
    // https://docs.rs/mailparse/0.13.0/mailparse/struct.ParsedMail.html
    // subparts: Vec<ParsedMail<'a>>
    // The subparts of this message or subpart. This vector is only non-empty if ctype.mimetype starts with "multipart/".
    let text_mail = if parsed_mail.subparts.is_empty() {
        &parsed_mail
    } else {
        parsed_mail
            .subparts
            .iter()
            .find(|&x| x.ctype.mimetype == "text/plain")
            .ok_or("no text/plain parts")?
    };
    let body = text_mail.get_body()?.trim_end().to_string();

    Ok(MyMessage {
        from,
        subject,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mailbox = MyMailbox {
            host: "ホスト名",
            user: "ユーザー名",
            password: "パスワード",
            ..Default::default()
        };

        let messages = read_mail(&mailbox);
        assert!(messages.is_ok());
        for message in messages.unwrap().iter() {
            println!("message: {:?}", message);
        }
    }
}
