// Copyright (C) 2024 Red Hat
// SPDX-License-Identifier: Apache-2.0

// #![warn(missing_docs)]

//! This library provides helper functions to query koji API.
//!
//! Note that instead of using the xmlrpc endpoints, this crate parses the HTML output.
//! See this issue for more detail: https://pagure.io/koji/issue/4007

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// A package name release version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NVR {
    name: Box<str>,
    version: Box<str>,
    release: Box<str>,
}

/// A build architecture like noarch or x86-64.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Arch(pub Box<str>);

/// A failed build.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KojiFailedBuild {
    pub package: NVR,
    pub arch: Arch,
    pub logs: Vec<Url>,
}

/// A complete build.
pub struct KojiBuild {
    pub package: NVR,
    pub logs: HashMap<Arch, Vec<Url>>,
}

/// A koji task.
pub enum KojiPage {
    Failed(KojiFailedBuild),
    Success(KojiBuild),
}

pub fn get_koji_build(agent: &ureq::Agent, url: &Url) -> Result<KojiPage, String> {
    let html = get_url(agent, url)?;
    match parse_result_failure(&html) {
        Ok(f) => Ok(KojiPage::Failed(f)),
        _ => parse_result_success(&html).map(KojiPage::Success),
    }
}

/*
pub fn get_build_arch_result(url: &Url) -> Result<KojiBuildArch, String> {
    // Parse URL
    todo!()
}

pub fn get_last_build(failed: &KojiBuildArch) -> Result<KojiBuildArch, String> {
    // Call https://koji.fedoraproject.org/koji/search?match=glob&type=package&terms=mingw-qt6-qt5compat
    // Then https://koji.fedoraproject.org/koji/buildinfo?buildID=2398878
    todo!()
}
*/

fn get_method_name<'a>(tree: &tree_sitter::Tree, body: &'a [u8]) -> Result<&'a str, String> {
    // This query matches '<th>Method</th><td>buildArch</td>'
    let query = tree_sitter::Query::new(
        tree.language(),
        r#"
(element (start_tag (tag_name))
  (element (start_tag (tag_name)) (text) @key (#eq? @key "Method") (end_tag (tag_name)))
  (element (start_tag (tag_name)) (text) @value (end_tag (tag_name)))
(end_tag (tag_name)))
"#,
    )
    .map_err(|e| format!("Query error {e}"))?;

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), body);
    if let Some(m) = matches.next() {
        match m.captures {
            [_k, v] => v
                .node
                .utf8_text(body)
                .map_err(|e| format!("Decode error {e}")),
            _ => Err("Bad captures".into()),
        }
    } else {
        Err("Couldn't find method name".into())
    }
}

fn get_arch_pkg<'a>(
    tree: &tree_sitter::Tree,
    body: &'a [u8],
) -> Result<(&'a str, &'a str), String> {
    // This query matches '<th>Parameters</th><td>...</td>'
    let query = tree_sitter::Query::new(
        tree.language(),
        r#"
(element (start_tag (tag_name))
  (element (start_tag (tag_name)) (text) @key (#eq? @key "Parameters") (end_tag (tag_name)))
  (element (start_tag (tag_name)) (_)+ @values (end_tag (tag_name)))
(end_tag (tag_name)))
"#,
    )
    .map_err(|e| format!("Query error {e}"))?;

    let mut cursor = tree_sitter::QueryCursor::new();
    // let mut arch = None;
    for m in cursor.matches(&query, tree.root_node(), body) {
        let mut iter = m.captures.into_iter();
        while let Some(c) = iter.next() {
            match c.node.utf8_text(body) {
                Ok("<strong>Arch</strong>") => {
                    if let Some(arch) = iter.next() {
                        println!("Got arch! {:#?}", arch.node.utf8_text(body));
                    }
                }
                r => println!("HERE {:#?}", r),
            }
        }
    }
    Err("Couldn't find method name".into())
}

fn parse_result_success(tree: &Html) -> Result<KojiBuild, String> {
    Err("oops".into())
}

fn parse_result_failure(html: &Html) -> Result<KojiFailedBuild, String> {
    let method = get_method_name(&html.tree, html.body.as_bytes())?;
    if method != "buildArch" {
        Err(format!("Only buildArch task are supported, found {method}"))
    } else {
        Ok(())
    }?;

    let arch = get_arch_pkg(&html.tree, html.body.as_bytes())?;

    Err("oops".into())
}

#[test]
fn test_parse_koji_page() {
    parse_koji_page(
        r#"
  <table>
    <tr>
      <th>ID</th><td>112975921</td>
    </tr>
    <tr>
      <th>Method</th><td>buildArch</td>
    </tr>
    <tr>
      <th>Parameters</th>
      <td>
        <strong>Arch</strong>: x86_64
<br/>
        <strong>Build tag</strong>: <a href="taginfo?tagID=71278">f40-build</a>
<br/>
        <strong>Keep srpm</strong>: True
<br/>
        <strong>Pkg</strong>: tasks/5819/112975819/bcc-0.29.1-1.fc40.src.rpm
<br/>
          <strong>Options:</strong><br/>
    &nbsp;&nbsp;repo_id&nbsp;=&nbsp;5797055
<br/>


      </td>
    </tr>
    <tr>
      <th>State</th>
      <td class="taskfailed">failed
      </td>
    </tr>
    <tr>
      <th>Created</th><td>Mon, 05 Feb 2024 14:01:44 UTC</td>
    </tr>
    <tr>
      <th>Started</th><td>Mon, 05 Feb 2024 14:02:22 UTC</td>
    <tr>
      <th>Completed</th><td>Mon, 05 Feb 2024 14:03:12 UTC</td>
    </tr>
    <tr>
      <th title="From task's creation">Total time</th>
      <td>0:01:28</td>
    </tr>
    <tr>
      <th title="From task's start">Task time</th>
      <td>0:00:50</td>
    </tr>
    <tr>
      <th>Owner</th>
      <td>
          <a href="userinfo?userID=4916">jmarchan</a>
      </td>
    </tr>
    <tr>
      <th>Channel</th>
      <td>
        <a href="channelinfo?channelID=1">default</a>
      </td>
    </tr>
    <tr>
      <th>Host</th>
      <td>
        <a href="hostinfo?hostID=496">buildhw-x86-07.iad2.fedoraproject.org</a>
      </td>
    </tr>
    <tr>
      <th>Arch</th><td>x86_64</td>
    </tr>
    <tr>
      <th>Buildroot</th>
      <td>
        <a href="buildrootinfo?buildrootID=48775034">/var/lib/mock/f40-build-48775034-5797055</a><br/>
      </td>
    </tr>
    <tr>
      <th>Parent</th>
        <td>
        <a href="taskinfo?taskID=112975817" class="taskfailed">build (rawhide, /rpms/bcc.git:110e48716fb75c07bd75ff97aa464bf90572d387)</a>
      </td>
    </tr>
    <tr>
      <th>Descendants</th>
      <td class="tree">

      </td>
    </tr>
    <tr>
      <th>Waiting?</th><td>no</td>
    </tr>
    <tr>
      <th>Awaited?</th><td>no</td>
    </tr>
    <tr>
      <th>Priority</th><td>19</td>
    </tr>
    <tr>
      <th>Weight</th><td>1.84</td>
    </tr>
    <tr>
      <th>Result</th>
      <td>
         <div id="result">
        <pre>BuildError: error building package (arch x86_64), mock exited with status 30; see root.log for more information</pre>
        </div>
      </td>
    </tr>
    <tr>
      <th>Output</th>
      <td>
        <a href="https://kojipkgs.fedoraproject.org//work/tasks/5921/112975921/build.log">build.log</a>
           (<a href="getfile?taskID=112975921&volume=DEFAULT&name=build.log&offset=-4000">tail</a>)
        <br/>
        <a href="https://kojipkgs.fedoraproject.org//work/tasks/5921/112975921/hw_info.log">hw_info.log</a>
           (<a href="getfile?taskID=112975921&volume=DEFAULT&name=hw_info.log&offset=-4000">tail</a>)
        <br/>
        <a href="https://kojipkgs.fedoraproject.org//work/tasks/5921/112975921/mock_output.log">mock_output.log</a>
           (<a href="getfile?taskID=112975921&volume=DEFAULT&name=mock_output.log&offset=-4000">tail</a>)
        <br/>
        <a href="https://kojipkgs.fedoraproject.org//work/tasks/5921/112975921/root.log">root.log</a>
           (<a href="getfile?taskID=112975921&volume=DEFAULT&name=root.log&offset=-4000">tail</a>)
        <br/>
        <a href="https://kojipkgs.fedoraproject.org//work/tasks/5921/112975921/state.log">state.log</a>
           (<a href="getfile?taskID=112975921&volume=DEFAULT&name=state.log&offset=-4000">tail</a>)
        <br/>
      </td>
    </tr>
  </table>
"#,
    );
    assert_eq!(true, false);
}

// Helper to query the api.
fn get_url(agent: &ureq::Agent, url: &Url) -> Result<Html, String> {
    agent
        .get(url.as_str())
        .call()
        .map_err(|e| format!("Request failed {}", e))?
        .into_string()
        .map_err(|e| format!("Bad response {}", e))
        .and_then(parse_html)
}

struct Html {
    tree: tree_sitter::Tree,
    body: String,
}

fn parse_html(body: String) -> Result<Html, String> {
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_html::language();
    parser
        .set_language(lang)
        .expect("Error loading HTML grammar");
    let tree = parser.parse(&body, None).ok_or("Couldn't parse body")?;
    Ok(Html { tree, body })
}
