use crate::config::RouteConfig;

#[derive(Clone)]
pub struct Router {
    routes: Vec<RouteConfig>,
}

impl Router {
    pub fn new(routes: Vec<RouteConfig>) -> Self {
        Self { routes }
    }

    pub fn match_route<'a>(&'a self, path: &str, method: &str) -> Option<&'a RouteConfig> {
        for route in &self.routes {
            if path.starts_with(&route.path_prefix) {
                if let Some(methods) = &route.methods {
                    if !methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
                        continue;
                    }
                }
                return Some(route);
            }
        }
        None
    }
}
