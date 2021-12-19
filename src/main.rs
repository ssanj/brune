use nom::{IResult, bytes::complete::{tag, take_while}, combinator::{map, opt}, sequence::delimited};


#[derive(Debug,PartialEq)]
struct HexValue(String);

#[derive(Debug,PartialEq)]
enum GitHubBranchType {
    Active,
    Deleted
}


#[derive(Debug,PartialEq)]
struct GitHubBranchLine {
    branch_name: String,
    branch_type: GitHubBranchType,
    comment: String
}


fn is_alphabetic(c: char) -> bool {
    c.is_alphabetic()
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn is_allowed_punctuation(c: char) -> bool {
    c == '-' || c == '_' || c == '/'
}

fn is_digit(c: char) -> bool {
    c.is_digit(10)
}


fn take_tag<'a>(prefix: &'a str, input: &'a str) -> IResult<&'a str, &'a str> {
    tag(prefix)(input)
}


fn take_whitespace<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    take_while(is_whitespace)(input)
}


fn take_alphabetic<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    take_while(is_alphabetic)(input)
}


fn take_branch_name<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    take_while(|c| is_alphabetic(c) || is_allowed_punctuation(c) || is_digit(c))(input)
}

fn take_annotation<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    delimited(
        tag("["),
        take_while(|c| is_alphabetic(c) || is_allowed_punctuation(c) || is_digit(c) || is_whitespace(c)),
        tag("]")
    )(input)
}

// TODO: How can we write this in terms of other Parsers instead of creating a new one?
fn take_whitespace_or_star<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    take_while(|c| is_whitespace(c) || c == '*')(input)
}

fn take_hex<'a>(input: &'a str) -> IResult<&'a str, HexValue> {
    map(take_while(is_hex_digit), |hv: &str| HexValue(hv.to_string()))(input)
}

/// Possible variations:
/// "[info]   FeatureA         dddeeee Random weird comments"
/// "[info]   FeatureD         ffff1111 [Ahead 1] Random weird comments"
/// "[info]   FeatureB         eeee3333 [behind 3] Random weird comments"
/// "[info] * master           0000bbbb [behind 2] Random weird comments"
/// "[info]   FeatureC         dddd3333 [gone] Random weird comments"
/// "[info]   PERSON1/FeatureD eeee4444 [gone] Random weird comments"
///
fn git_line_parser<'a>(input: &'a str) -> IResult<&'a str, GitHubBranchLine> {
    let (tail2, _)          = take_whitespace_or_star(input)?;
    let (tail3, branch_n)   = take_branch_name(tail2)?;
    let (tail4, _)          = take_whitespace(tail3)?;
    let (tail5, _hex_value) = take_hex(tail4)?;
    let (tail6, _)          = take_whitespace(tail5)?;
    let (tail7, gone_op)    = opt(|i: &'a str| take_annotation(i))(tail6)?;
    let (tail8, _)          = opt(|i: &'a str| take_whitespace(i))(tail7)?;

    let branch_type = match gone_op {
        Some("gone") => GitHubBranchType::Deleted,
        _ => GitHubBranchType::Active
    };

    let branch_name = branch_n.to_string();
    let comment = tail8.clone().to_string();

    // TODO: We don't need to return tail8 here as we are done.
    let pair = (tail8 ,GitHubBranchLine { branch_name, branch_type, comment });

    Ok(pair)

}

fn main() {
    let git_line = "   PERSON1/FeatureD eeee4444 [gone] Random weird comments";
    println!("parsing '{}'", git_line);
    let (_, branch_line) = git_line_parser(git_line).unwrap();
    println!("{:?}", branch_line)
}

#[test]
fn parse_git_line_remove_info() {
    let git_line = "[info]abc";
    let (r, m) = take_tag("[info]", git_line).unwrap();

    assert_eq!(m, "[info]");
    assert_eq!(r, "abc");
}

#[test]
fn parse_git_line_remove_whitespace() {
    let git_line = "   FeatureC  abcd";
    let (r, m) = take_whitespace(git_line).unwrap();

    assert_eq!(m, "   ");
    assert_eq!(r, "FeatureC  abcd");
}

#[test]
fn parse_git_line_take_alphabetics() {
    let git_line = "FeatureC         dddd3333";
    let (r, m) = take_alphabetic(git_line).unwrap();
    assert_eq!(m, "FeatureC");
    assert_eq!(r, "         dddd3333");
}

/// Branch name with dashes and slashes
#[test]
fn parse_git_line_take_branch_name() {
    let git_line = "xyz/some-name-with-dashes         dddd3333";
    let (r, m) = take_branch_name(git_line).unwrap();
    assert_eq!(m, "xyz/some-name-with-dashes");
    assert_eq!(r, "         dddd3333");
}

#[test]
fn parse_git_line_take_branch_name_2() {
    let git_line = "ID-9AB-blee-blah-2                              dddd3333 Blah de blah";
    let (r, m) = take_branch_name(git_line).unwrap();
    assert_eq!(m, "ID-9AB-blee-blah-2");
    assert_eq!(r, "                              dddd3333 Blah de blah");
}

#[test]
fn parse_git_line_take_hex() {
    let git_line = "dddd3333G32H";
    let (r, m) = take_hex(git_line).unwrap();
    assert_eq!(m, HexValue("dddd3333".to_string()));
    assert_eq!(r, "G32H");
}


/// 1. Alphabetic branch name
/// 2. [gone] annotation
#[test]
fn parse_git_line() {
    let git_line = "   FeatureC         dddd3333 [gone] Random weird comments";
    let (r, m) = git_line_parser(git_line).unwrap();
    let expected = GitHubBranchLine { branch_name: "FeatureC".to_string(), branch_type: GitHubBranchType::Deleted, comment: "Random weird comments".to_string() };
    assert_eq!(m,  expected);
    assert_eq!(r, "Random weird comments");
}

/// 1. hyphenated branch name
/// 2. No annotation
#[test]
fn parse_git_line_2() {
    let git_line = "   ID-9AB-blee-blah-2                              dddd3333 Blah de blah";
    let (r, m) = git_line_parser(git_line).unwrap();
    let expected = GitHubBranchLine { branch_name: "ID-9AB-blee-blah-2".to_string(), branch_type: GitHubBranchType::Active, comment: "Blah de blah".to_string() };
    assert_eq!(m,  expected);
    assert_eq!(r, "Blah de blah");
}

/// 1. Hyphenated branch name
/// 2. Star (representing current branch)
/// 3. No annotation
#[test]
fn parse_git_line_3() {
    let git_line = " * ID-9AB-blee-blah-2                              dddd3333 Blah de blah";
    let (r, m) = git_line_parser(git_line).unwrap();
    let expected = GitHubBranchLine { branch_name: "ID-9AB-blee-blah-2".to_string(), branch_type: GitHubBranchType::Active, comment: "Blah de blah".to_string() };
    assert_eq!(m,  expected);
    assert_eq!(r, "Blah de blah");
}

/// 1. Alphabetic branch name
/// 2. [behind 3] annotation
#[test]
fn parse_git_line_4() {
    let git_line = "FeatureB         eeee3333 [behind 3] Random weird comments";
    let (r, m) = git_line_parser(git_line).unwrap();
    let expected = GitHubBranchLine { branch_name: "FeatureB".to_string(), branch_type: GitHubBranchType::Active, comment: "Random weird comments".to_string() };
    assert_eq!(m,  expected);
    assert_eq!(r, "Random weird comments");
}


/// 1. Hyphenated and slashed branch name
/// 2. Star (representing current branch)
/// 3. [ahead 1] annotation
#[test]
fn parse_git_line_5() {
    let git_line = " * XYZ/ID-9AB-blee-blah-2                        dddd3333   [ahead 1]   Blah de blah";
    let (r, m) = git_line_parser(git_line).unwrap();
    let expected = GitHubBranchLine { branch_name: "XYZ/ID-9AB-blee-blah-2".to_string(), branch_type: GitHubBranchType::Active, comment: "Blah de blah".to_string() };
    assert_eq!(m,  expected);
    assert_eq!(r, "Blah de blah");
}

/// 1. Hyphenated and slashed branch name
/// 2. Star (representing current branch)
/// 3. [ahead 1] annotation
/// 4. Emoji in comment
#[test]
fn parse_git_line_6() {
    let git_line = " * XYZ/ID-9AB-blee-blah-2                        dddd3333   [ahead 1]   Blah 😃 blah";
    let (r, m) = git_line_parser(git_line).unwrap();
    let expected = GitHubBranchLine { branch_name: "XYZ/ID-9AB-blee-blah-2".to_string(), branch_type: GitHubBranchType::Active, comment: "Blah 😃 blah".to_string() };
    assert_eq!(m,  expected);
    assert_eq!(r, "Blah 😃 blah");
}
