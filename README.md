# imap と mailparse でメールを読む

## 手順
[src/lib.rs](src/lib.rs) のテスト部を改修して、  
`$ cargo test -- --nocapture`

所望のメールが出力されればOK

## 泣き所
それにしても「?」「unwrap」「ok_or」ばかりで頭がおかしくなりそう・・・  
もう少しきれいに書く方法はないものでしょうか？