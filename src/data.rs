use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};


pub struct PHTypeAttributes {
    enumerated: bool,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum PlaceholderType {
    UNKNOWN, // from Google protobuf style guide, but is this necessary? I think not
    GENDER,
    PLURAL,
    OTHER(String) // let this be open-ended for all the things we know and
                  // all the future needs we can't predict.
}

// return a map with pre-defined meta-information about common PlaceholderTypes.
// I think this is useful, esp for an l10n/TMS system.  But I haven't decided
// if/how it relates strictly to message formatting itself, so unil then,
// this will actually be unused.  If this is useful, then the user can extend
//  this map to support whatever custom PH types that the user chooses to use
// in `Placeholder.OTHER(String)`.
pub fn ph_type_attrs_map() -> HashMap<PlaceholderType, PHTypeAttributes> {
    let mut m = HashMap::new();
    m.insert(
        PlaceholderType::GENDER,
        PHTypeAttributes{ enumerated: true }
    );
    m.insert(
        PlaceholderType::PLURAL,
        PHTypeAttributes{ enumerated: true }
    );
    m
}

pub struct Placeholder {
    // id & name for PH, used for val interpolation in the formatted string.
    // Let the user decide whether this should be unique or shared
    // across multiple PH instances within the same message.
    // Ex: if PH represents a product name, then maybe you want to
    // call it PRODUCT_NAME everywhere that same string is reoccurs
    // in the message.
    // Ex: if you have multiple inline <span> tags that need to be
    // turned into placeholders, then you might want to use SPAN1,
    // SPAN2, ... to indicate that the contents may very well differ.
    // and <b> and <i> tags may just all be B and I because they are
    // semantically same, and therefore interchangeable.
    id: String,

    // type of the PH.
    // See notes for PlaceholderType for nuances of PH types.
    ph_type: PlaceholderType,

    // a user-supplied text representation of the PH, if available.
    // For PHs that are created by the user (or user's l10n tool),
    // taking a source document, parsing text units out of the doc file format, 
    // and replacing inline non-translatable content in each TU with a PH,
    // we already know the text that the PH is "holding the place" for.
    // If not present, then the value must be present in the map 
    // `SingleMessage.ph_vals` that is keyed by this PH's `Placeholder.id`.
    default_text_val: Option<String>,
}

impl fmt::Display for Placeholder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{{{}}}", self.id))
    }
}

// PHValsMap indicates how to uniquely select a message,
// while still being extensible to include runtime values
// during the formatting phase.
#[derive(Clone, Eq, Debug)] // impl for Hash and PartialEq below
pub struct PHValsMap {
    map: HashMap<String, String>,
}

impl std::hash::Hash for PHValsMap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hasher = DefaultHasher::new();

        for (key, val) in &self.map {
            key.hash(&mut hasher);
            val.hash(&mut hasher);
        }

        hasher.finish();
    }
}

impl PartialEq for PHValsMap {
    fn eq(&self, other: &PHValsMap) -> bool {
        &self.map == &other.map
    }
}

impl fmt::Display for PHValsMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entries = &self.map.iter();
        let entry_strs: &Vec<String> =
            (&self.map
                .iter()
                .map(|(k,v)| format!("{}:{}", k.clone(), v.clone()))
            .collect::<Vec<String>>());
        let entry_list_str = entry_strs.join(", ");
        let result_str = format!("{{{}}}", entry_list_str);
        write!(f, "{}", format!("{}", result_str))
    }
}

pub struct TextPart {
    text: String,
}

impl fmt::Display for TextPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{}", self.text))
    }
}

pub enum PatternPart {
    TEXTPART(TextPart),
    PLACEHOLDER(Placeholder),
}

impl fmt::Display for PatternPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = match &self {
            PatternPart::TEXTPART(text_part) => {
                write!(f, "{}", format!("{}", text_part))
            }
            PatternPart::PLACEHOLDER(placeholder) => {
                write!(f, "{}", format!("{}", placeholder))
            }
        };
        result
    }
}

pub struct MessagePattern {
    parts: Vec<PatternPart>
}

impl fmt::Display for MessagePattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let part_strs: Vec<String> = self.parts.iter().map(|part| format!("{}", part)).collect();
        let pattern_str = part_strs.join("");
        write!(f, "{}", format!("[{}]", pattern_str))
    }
}

pub struct SingleMessage {
    // unique id for the SingleMessage, globally unique.
    id: String,

    locale: String,
    pattern: MessagePattern,
    ph_vals: PHValsMap, // type of value should prob be Any
}

impl fmt::Display for SingleMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{}", self.pattern))
    }
}

pub struct MessageGroup {
    id: String,
    messages: HashMap<PHValsMap, SingleMessage>,
}

impl fmt::Display for MessageGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let line1 = format!("{}: {{", &self.id);
        let last_line = format!("}}");
        let entries = &self.messages.iter();
        let mut entry_strs: &Vec<String> =
            (&self.messages
                .iter()
                .map(|(k,v)| format!("  {}: {}", k.clone(), v.clone()))
            .collect::<Vec<String>>());

        let mut all_lines: Vec<String> = Vec::new();
        all_lines.append(&mut vec![line1]);
        all_lines.extend((*entry_strs).clone());
        all_lines.append(&mut vec![last_line]);
        let result_str = all_lines.join("\n");

        write!(f, "{}", format!("{}", result_str))
    }
}

pub enum MessageType {
    SINGLE(SingleMessage),
    GROUP(MessageGroup)
}

pub struct TextUnit {
    src: MessageType,
    tgt: MessageType,
}


//
// unit tests
// 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ph_vals_map_hasheq() {
        let mut map1 = HashMap::new();
        &map1.insert(String::from("COUNT"), String::from("5"));
        let ph_vals1 =
         PHValsMap {
             map: map1,
            };

        let mut map2 = HashMap::new();
        &map2.insert(String::from("COUNT"), String::from("14"));
        let ph_vals2 =
         PHValsMap {
             map: map2,
            };

        let mut map3 = HashMap::new();
        &map3.insert(String::from("COUNT"), String::from("5"));
        let ph_vals3 =
            PHValsMap {
                map: map3,
            };

        assert_eq!(&ph_vals1, &ph_vals3);
        assert_ne!(&ph_vals1, &ph_vals2);
        assert_ne!(&ph_vals2, &ph_vals3);

        let mut map4 = HashMap::new();
        &map4.insert(String::from("count"), String::from("5"));
        &map4.insert(String::from("COUNT"), String::from("5"));
        let ph_vals4 =
            PHValsMap {
                map: map4,
            };

        assert_ne!(&ph_vals1, &ph_vals4);
        assert_ne!(&ph_vals3, &ph_vals4);
    }

    #[test]
    fn test_construct_message() {
        // `PHValsMap`s for selecting specific messages.
        let ph_vals1 = PHValsMap{ map: {
            let mut m = HashMap::default();
            m.insert(String::from("COUNT"), String::from("=0"));
            m
        }};
        let ph_vals2 = PHValsMap{ map: {
            let mut m = HashMap::default();
            m.insert(String::from("COUNT"), String::from("ONE"));
            m
        }};
        let ph_vals3 = PHValsMap{ map: {
            let mut m = HashMap::default();
            m.insert(String::from("COUNT"), String::from("OTHER"));
            m
        }};

        let msg1 = SingleMessage {
            id: String::from("msg1"),
            locale: String::from("en"),
            pattern: MessagePattern{ 
                parts: vec![
                    PatternPart::TEXTPART(TextPart{ text: String::from("No items selected.") }),
                ],
            },
            ph_vals: ph_vals1.clone(),
        };
        let msg2 = SingleMessage {
            id: String::from("msg2"),
            locale: String::from("en"),
            pattern: MessagePattern{ 
                parts: vec![
                    PatternPart::PLACEHOLDER(Placeholder{
                        id: String::from("COUNT"),
                        ph_type: PlaceholderType::PLURAL,
                        default_text_val: Option::None,
                     }),
                    PatternPart::TEXTPART(TextPart{ text: String::from(" item selected.") }),
                ],
            },
            ph_vals: ph_vals2.clone(),
        };
        let msg3 = SingleMessage {
            id: String::from("msg3"),
            locale: String::from("en"),
            pattern: MessagePattern{ 
                parts: vec![
                    PatternPart::PLACEHOLDER(Placeholder{
                        id: String::from("COUNT"),
                        ph_type: PlaceholderType::PLURAL,
                        default_text_val: Option::None,
                     }),
                    PatternPart::TEXTPART(TextPart{ text: String::from(" items selected.") }),
                ],
            },
            ph_vals: ph_vals3.clone(),
        };

        let msg_grp_key_1 = ph_vals1.clone();
        let msg_grp_key_2 = ph_vals2.clone();
        let msg_grp_key_3 = ph_vals3.clone();

        let mut messages: HashMap<PHValsMap, SingleMessage> = HashMap::new();
        messages.insert(msg_grp_key_1, msg1);
        messages.insert(msg_grp_key_2, msg2);
        messages.insert(msg_grp_key_3, msg3);

        println!("msg1: {}", &messages.get(&ph_vals1).unwrap());
        println!("msg2: {}", &messages.get(&ph_vals2).unwrap());
        println!("msg3: {}", &messages.get(&ph_vals3).unwrap());

        let msg_grp = MessageGroup {
            id: String::from("msg_grp"),
            messages,
        };

        println!("msg_grp =");
        println!("{}", msg_grp);
    }
}