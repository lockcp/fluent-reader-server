use jieba_rs::Jieba;
use lazy_static::lazy_static;
use serde_json::json;
use std::collections::HashSet;
use std::iter::FromIterator;
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

lazy_static! {
    static ref STOP_CHARS: HashSet<&'static str> = HashSet::from_iter(
        "!\"#$%&'()*+,-./:;<=>?@[\\]^_{|}~`。？！，、；：“ ” ‘ ’「」『』（）【】—…-～	
《》〈〉_ "
            .split("")
    );
}

pub fn get_unique_words(words: &Vec<&str>) -> (serde_json::Value, usize) {
    let mut unique_words = json!({});
    let mut total_word_count = 0usize;

    let map = match unique_words {
        serde_json::Value::Object(ref mut map) => map,
        _ => panic!("unique_words serde_json::Value isn't an Object!"),
    };

    for word in words.iter() {
        let lowercase = word.to_lowercase();
        if !STOP_CHARS.contains(&lowercase[..]) {
            match map.get(&lowercase) {
                Some(num_val) => {
                    let new_num = num_val.as_i64().unwrap() + 1i64;
                    map.insert(lowercase, json!(new_num));
                    total_word_count = total_word_count + 1;
                }
                None => {
                    map.insert(lowercase, json!(1));
                    total_word_count = total_word_count + 1;
                }
            };
        }
    }

    (unique_words, total_word_count)
}

pub fn get_or_query_string(
    string_opt: &Option<String>,
    lang_opt: &Option<String>,
) -> Option<String> {
    match string_opt {
        Some(ref string) => match lang_opt {
            Some(ref lang) => Some(
                get_words(&string[..], &lang[..])
                    .iter()
                    .filter_map(|&word| match word {
                        " " => None,
                        _ => Some(word),
                    })
                    .collect::<Vec<&str>>()
                    .join(" OR "),
            ),
            None => None,
        },
        None => None,
    }
}
