use std::error;
use std::vec::Vec;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    /// Track what the current highlighted item in the list is.
    pub current_request_index: usize,

    /// Temporary list of requests
    pub requests: Vec<DummyRequest>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_request_index: 0,
            requests: dummy_http_data(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_list_index(&mut self) {
        if let Some(res) = self.current_request_index.checked_add(1) {
            self.current_request_index = res % self.requests.len();
        }
    }

    pub fn decrement_list_index(&mut self) {
        if self.current_request_index == 0 {
            self.current_request_index = self.requests.len();
        }

        if let Some(res) = self.current_request_index.checked_sub(1) {
            self.current_request_index = res;
        }
    }
}

#[derive(Debug, Clone)]
pub struct DummyRequest {
    pub domain: String,
    pub verb: String,
    pub request_body: String,
    pub response_body: String,
}

fn dummy_http_data() -> Vec<DummyRequest> {
    vec![
        DummyRequest {
            domain: "hackerone.com/foo/bar/baz".to_owned(),
            verb: "GET".to_owned(),
            request_body: "
GET /zork?type=team HTTP/1.1
Host: hackerone.com

Cookie: h1_device_id=178f6f86; __Host-session=dEt6aEtQ08...;
User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8
Accept-Language: en-US,en;q=0.5
Accept-Encoding: gzip, deflate, br
Upgrade-Insecure-Requests: 1
Sec-Fetch-Dest: document
Sec-Fetch-Mode: navigate
Sec-Fetch-Site: none
Sec-Fetch-User: ?1
Dnt: 1
Sec-Gpc: 1
Te: trailers
Connection: close
".to_owned()
            ,

            response_body: "
HTTP/2 200 OK

Content-Type: text/html; charset=utf-8
Cache-Control: no-store
Content-Disposition: inline; filename='response.html'
X-Request-Id: 30125139-8cbf-4cc3-bb1f-f98f86caec3c
Set-Cookie: __Host-session=bE5hSz...
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-Xss-Protection: 1; mode=block
X-Download-Options: noopen
X-Permitted-Cross-Domain-Policies: none
Referrer-Policy: strict-origin-when-cross-origin
Expect-Ct: enforce, max-age=86400
Content-Security-Policy: default-src 'none'; base-uri 'self'; block-all-mixed-content; child-src www.yout...
Cf-Cache-Status: DYNAMIC
Server: cloudflare
Cf-Ray: 825282af6b029872-SJC

Here haz content
".to_owned(),
        },

        DummyRequest {
            domain: "https://example.com/foobar".to_owned(),
            verb: "POST".to_owned(),
            request_body: "
POST example.com/foobar HTTP/1.1
Host: example.com

Cookie: h1_device_id=178f6f86; __Host-session=dEt6aEtQ08...;
User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8
Accept-Language: en-US,en;q=0.5
Accept-Encoding: gzip, deflate, br
Upgrade-Insecure-Requests: 1
Connection: close
".to_owned()
            ,

            response_body: "
HTTP/2 200 OK

Content-Type: text/html; charset=utf-8
Cache-Control: no-store
Content-Disposition: inline; filename='response.html'
X-Request-Id: 30125139-8cbf-4cc3-bb1f-f98f86caec3c
Set-Cookie: __Host-session=bE5hSz...
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
Referrer-Policy: strict-origin-when-cross-origin
Expect-Ct: enforce, max-age=86400
Cf-Ray: 825282af6b029872-SJC

Moar content
".to_owned(),
        },
        DummyRequest {
            domain: "https://foobar.com/hello".to_owned(),
            verb: "PUT".to_owned(),
            request_body: "
PUT /hello HTTP/1.1
Host: foobar.com


Sec-Fetch-Dest: document
Sec-Fetch-Mode: navigate
Sec-Fetch-Site: none
Sec-Fetch-User: ?1
Dnt: 1
Sec-Gpc: 1
Te: trailers
Connection: close
".to_owned()
            ,

            response_body: "
HTTP/2 200 OK

Content-Type: text/html; charset=utf-8
Cache-Control: no-store
Content-Disposition: inline; filename='response.html'
X-Request-Id: 30125139-8cbf-4cc3-bb1f-f98f86caec3c
X-Clacks-Overhead: GNU Terry Pratchett
Expect-Ct: enforce, max-age=86400
Content-Security-Policy: default-src 'none'; base-uri 'self'; block-all-mixed-content; child-src www.yout...
Cf-Cache-Status: DYNAMIC
Server: cloudflare
Cf-Ray: 825282af6b029872-SJC

Here haz content
".to_owned(),
        },
        DummyRequest {
            domain: "https://api.github.com/repos/octocat/Spoon-Knife/issues".to_owned(),
            verb: "POST".to_owned(),
            request_body: "
POST /repos/octocat/Spoon-Knife/issues HTTP/1.1
Host: api.github.com

Accept-Encoding: gzip, deflate, br
Upgrade-Insecure-Requests: 1
Sec-Fetch-Dest: document
Dnt: 1
Sec-Gpc: 1
Te: trailers
Connection: close
".to_owned()
            ,

            response_body: "
HTTP/2 200 OK

Content-Type: application/json; charset=utf-8
Cache-Control: no-store
Cf-Cache-Status: DYNAMIC
Server: cloudflare
Cf-Ray: 825282af6b029872-SJC

[
  {
    'id': 1,
    'node_id': 'MDU6SXNzdWUx',
    'url': 'https://api.github.com/repos/octocat/Hello-World/issues/1347',
    'repository_url': 'https://api.github.com/repos/octocat/Hello-World',
    'labels_url': 'https://api.github.com/repos/octocat/Hello-World/issues/1347/labels{/name}',
    'comments_url': 'https://api.github.com/repos/octocat/Hello-World/issues/1347/comments',
    'events_url': 'https://api.github.com/repos/octocat/Hello-World/issues/1347/events',
    'html_url': 'https://github.com/octocat/Hello-World/issues/1347',
    'number': 1347,
    'state': 'open',
    'title': 'Found a bug',
    'body': 'I'm having a problem with this.',
    'user': {
      'login': 'octocat',
      'id': 1,
      'node_id': 'MDQ6VXNlcjE=',
      'avatar_url': 'https://github.com/images/error/octocat_happy.gif',
      'gravatar_id': '',
      'url': 'https://api.github.com/users/octocat',
      'html_url': 'https://github.com/octocat',
      'followers_url': 'https://api.github.com/users/octocat/followers',
      'following_url': 'https://api.github.com/users/octocat/following{/other_user}',
      'gists_url': 'https://api.github.com/users/octocat/gists{/gist_id}',
      'starred_url': 'https://api.github.com/users/octocat/starred{/owner}{/repo}',
      'subscriptions_url': 'https://api.github.com/users/octocat/subscriptions',
      'organizations_url': 'https://api.github.com/users/octocat/orgs',
      'repos_url': 'https://api.github.com/users/octocat/repos',
      'events_url': 'https://api.github.com/users/octocat/events{/privacy}',
      'received_events_url': 'https://api.github.com/users/octocat/received_events',
      'type': 'User',
      'site_admin': false
    },
]

".to_owned(),
        },
        DummyRequest {
            domain: "https://api.zork.com/resource/foo".to_owned(),
            verb: "OPTIONS".to_owned(),
            request_body: "
OPTIONS /resource/foo
Access-Control-Request-Method: DELETE
Access-Control-Request-Headers: origin, x-requested-with
Origin: https://thegreat.underground.empire
".to_owned()
            ,

            response_body: "
HTTP/1.1 204 No Content
Connection: keep-alive
Access-Control-Allow-Origin: https://foo.bar.org
Access-Control-Allow-Methods: POST, GET, OPTIONS, DELETE
Access-Control-Max-Age: 86400
".to_owned(),
        },


    ]
}
