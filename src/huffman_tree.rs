use std::cmp::Reverse;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use priority_queue::PriorityQueue;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum HuffmanNode {
    Leaf{
        element: char,
    },
    Internal{
        left: Option<Box<HuffmanNode>>,
        right: Option<Box<HuffmanNode>>,
    },
}

fn create_huffman_tree(characters: HashMap<char, u32>) -> HuffmanNode {
    let mut queue = PriorityQueue::new();
    for (ch, weight) in characters.iter() {
        queue.push(
            HuffmanNode::Leaf { element: *ch },
            Reverse(*weight)
        );
    }
    while queue.len() > 1 {
        let (left, left_weight) = queue.pop().unwrap();
        let (right, right_weight) = queue.pop().unwrap();
        let parent = HuffmanNode::Internal {
            left: Some(Box::new(left)), right: Some(Box::new(right))
        };
        queue.push(parent, Reverse(left_weight.0 + right_weight.0));
    }
    let (root, _) = queue.pop().unwrap();
    root
}

fn create_huffman_table(root: HuffmanNode) -> HashMap<char, String> {
    let mut charmap = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back(("".to_string(), root));
    while !queue.is_empty() {
        let (s, node) = queue.pop_front().unwrap();
        match node {
            HuffmanNode::Internal { left, right } => {
                queue.push_back((s.clone() + "0", *left.unwrap()));
                queue.push_back((s + "1", *right.unwrap()));
            }
            HuffmanNode::Leaf { element } => {
                charmap.insert(element, s);
            }
        }
    }
    charmap
}

fn count_chars(contents: &str) -> HashMap<char, u32> {
    let mut map = HashMap::new();
    for ch in contents.chars() {
        map.entry(ch).and_modify(|counter| *counter += 1).or_insert(1);
    }
    map
}

pub fn encode(
    output_file: &mut File,
    contents: String
) {
    let charmap = count_chars(&contents);
    let tree = create_huffman_tree(charmap);
    let table = create_huffman_table(tree.clone());
    let mut byte: u8 = 0x00;
    let mut curr: i32 = 7;
    let mut stack = vec![tree];
    let mut output = Vec::new();
    while let Some(node) = stack.pop() {
        match node {
            HuffmanNode::Leaf { element } => {
                byte |= 0b1 << curr;
                curr -= 1;
                if curr < 0 {
                    output.push(byte);
                    curr = 7;
                    byte = 0x00;
                }
                for i in (0..32).rev() {
                    byte |= (((element as u32 >> i) & 0b1) as u8) << curr;
                    curr -= 1;
                    if curr < 0 {
                        output.push(byte);
                        curr = 7;
                        byte = 0x00;
                    }
                }
            }
            HuffmanNode::Internal { left, right } => {
                stack.push(*right.unwrap());
                stack.push(*left.unwrap());
                curr -= 1;
                if curr < 0 {
                    output.push(byte);
                    curr = 7;
                    byte = 0x00;
                }
            }
        }
    }
    for c in contents.chars() {
        let code = table.get(&c).unwrap();
        for digit in code.chars() {
            if digit == '1' {
                byte |= 0b1 << curr;
            }
            curr -= 1;
            if curr < 0 {
                output.push(byte);
                curr = 7;
                byte = 0x00;
            }
        }
    }
    let bits_remaining = 7 - curr;
    if bits_remaining < 6 {
        byte |= (bits_remaining + 2) as u8;
        output.push(byte);
    } else {
        output.push(byte);
        byte = ((bits_remaining + 2) % 8) as u8;
        output.push(byte);
    }
    if let Err(e) = output_file.write_all(&output) {
        panic!("Could not write encoded file: {}", e);
    }
}

pub fn decode(input_file: &mut File, output_file: &mut File) {
    let mut input = Vec::new();
    let _read = input_file.read_to_end(&mut input).expect("Couldn't read file to decode");
    let mut curr: i8 = 6;
    let mut index = 0;
    let mut root = HuffmanNode::Internal { left: None, right: None };
    let mut byte = input.get(index).expect("Input ended prematurely.");
    index += 1;
    assert_eq!((byte >> 7) & 1, 0);
    while has_space(&root) {
        if (byte >> curr) & 1 == 0 {
            insert(&mut root, HuffmanNode::Internal { left: None, right: None });
            curr -= 1;
            if curr < 0 {
                byte = input.get(index).expect("Input ended prematurely.");
                index += 1;
                curr = 7;
            }
        } else {
            curr -= 1;
            if curr < 0 {
                byte = input.get(index).expect("Input ended prematurely.");
                index += 1;
                curr = 7;
            }
            let mut element: u32 = 0;
            for offset in (0..32).rev() {
                element |= ((*byte as u32 >> curr) & 1) << offset;
                curr -= 1;
                if curr < 0 {
                    byte = input.get(index).expect("Input ended prematurely.");
                    index += 1;
                    curr = 7;
                }
            }
            insert(&mut root, HuffmanNode::Leaf { element: char::from_u32(element).unwrap() });
        }
    }
    let mut output = String::new();
    let last_byte = input.last().unwrap();
    let mut last_bits = (last_byte & 0x7) + 6;
    let last_index = if last_bits > 8 {
        last_bits -= 8;
        input.len() - 1
    } else {
        input.len() - 2
    };
    loop {
        let mut cursor = &root;
        loop {
            if (byte >> curr) & 1 == 0 {
                if let HuffmanNode::Internal { left, right: _ } = cursor {
                    cursor = left.as_ref().unwrap();
                }
                if let HuffmanNode::Leaf { element } = cursor {
                    output.push(*element);
                    curr -= 1;
                    if curr < 0 {
                        byte = input.get(index).expect("Input ended prematurely.");
                        index += 1;
                        curr = 7;
                    }
                    break;
                }
            } else {
                if let HuffmanNode::Internal { left: _, right } = cursor {
                    cursor = right.as_ref().unwrap();
                }
                if let HuffmanNode::Leaf { element } = cursor {
                    output.push(*element);
                    curr -= 1;
                    if curr < 0 {
                        byte = input.get(index).expect("Input ended prematurely.");
                        index += 1;
                        curr = 7;
                    }
                    break;
                }
            }
            curr -= 1;
            if curr < 0 {
                byte = input.get(index).expect("Input ended prematurely.");
                index += 1;
                curr = 7;
            }
        }
        if index > last_index && 7 - (curr as u8) >= last_bits {
            break;
        }
    }
    let _ = write!(output_file, "{output}");
}

fn insert(node: &mut HuffmanNode, to_insert: HuffmanNode) -> bool {
    match node {
        HuffmanNode::Internal { left, right } => {
            let inserted = match left {
                None => {
                    *left = Some(Box::new(to_insert.clone()));
                    true
                }
                Some(left_node) => {
                    insert(left_node, to_insert.clone())
                }
            };
            if inserted { return inserted; }
            match right {
                None => {
                    *right = Some(Box::new(to_insert));
                    true
                }
                Some(right_node) => {
                    insert(right_node, to_insert)
                }
            }
        }
        HuffmanNode::Leaf { element: _ } => false,
    }
}

fn has_space(node: &HuffmanNode) -> bool {
    match node {
        HuffmanNode::Internal { left, right } => {
            let space = match left {
                None => true,
                Some(left_node) => has_space(left_node),
            };
            if space { return space; }
            match right {
                None => true,
                Some(right_node) => has_space(right_node),
            }
        }
        HuffmanNode::Leaf { element: _ } => false,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn huffman_tree_creation() {
        use HuffmanNode::*;
        let map = HashMap::from([
            ('a', 15),
            ('b', 20),
            ('c', 30),
        ]);
        let root = create_huffman_tree(map);
        assert_eq!(root, Internal { left: Some(Box::new(Leaf { element: 'c' })), right: Some(Box::new(
            Internal { left: Some(Box::new(Leaf { element: 'a' })), right: Some(Box::new(Leaf { element: 'b' }))}
        ))});
    }

    #[test]
    fn huffman_table_creation() {
        let map = HashMap::from([
            ('e', 120),
            ('u', 37),
            ('d', 42),
            ('l', 41),
            ('c', 32),
            ('m', 24),
            ('z', 2),
            ('k', 7),
        ]);
        let root = create_huffman_tree(map);
        let table = create_huffman_table(root);
        assert_eq!(table.get(&'e'), Some(&"0".to_string()));
        assert_eq!(table.get(&'z'), Some(&"111100".to_string()));
    }
}
