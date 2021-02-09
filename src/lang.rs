use jieba_rs::Jieba;
use lazy_static::lazy_static;
use serde_json::json;
use unicode_segmentation::UnicodeSegmentation;

pub fn get_words<'a>(text: &'a str, lang: &str) -> Vec<&'a str> {
    match lang {
        "en" => get_words_english(text),
        "zh" => get_words_chinese(text),
        _ => panic!("Got unsupported language for get_words: {}", text),
    }
}

fn get_words_english<'a>(text: &'a str) -> Vec<&'a str> {
    text.split_word_bounds().collect::<Vec<&str>>()
}

lazy_static! {
    static ref JIEBA: Jieba = Jieba::new();
}

fn get_words_chinese<'a>(text: &'a str) -> Vec<&'a str> {
    JIEBA.cut(text, false)
}

pub fn get_unique_words(words: &Vec<&str>) -> serde_json::Value {
    let mut unique_words = json!({});

    let map = match unique_words {
        serde_json::Value::Object(ref mut map) => map,
        _ => panic!("unique_words serde_json::Value isn't an Object!"),
    };

    for word in words.iter() {
        let lowercase = word.to_lowercase();
        if map.contains_key(&lowercase) {
            let current_value = map.get(&lowercase).unwrap().clone();
            if let serde_json::Value::Number(num) = current_value {
                map.insert(lowercase, json!(num.as_i64().unwrap() + 1i64));
            } else {
                panic!("Value stored in unique words map is not a number!");
            }
        } else {
            map.insert(lowercase, json!(1));
        }
    }

    unique_words
}
