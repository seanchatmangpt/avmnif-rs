//! Deterministic, `no_std` swarm admission and route planning.
//!
//! This is the portable policy slice of `unrdf/packages/atomvm/swarm-cluster.mjs`.
//! It owns neither network connectivity nor raw authentication material. A route
//! is a selected path through admitted topology; it is not authority to execute.

use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Admitted AtomVM swarm metadata. `authority_ref` names externally-owned
/// authority; it deliberately does not contain a raw cookie or secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmDescriptor {
    pub id: String,
    pub gateway_node: String,
    pub authority_ref: String,
    pub endpoint: String,
}

/// Canonical undirected link between two admitted swarms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmLink {
    pub left: String,
    pub right: String,
}

/// Typed refusal before any network actuation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmRefusal {
    InvalidClusterId,
    InvalidSwarmId,
    InvalidGatewayNode,
    InvalidEndpoint,
    MissingAuthorityRef,
    DuplicateSwarm,
    UnknownSwarm,
    SelfLink,
    NoRoute,
}

/// Admitted topology plus deterministic route selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmTopology {
    cluster_id: String,
    swarms: Vec<SwarmDescriptor>,
    links: Vec<SwarmLink>,
}

impl SwarmTopology {
    pub fn new(cluster_id: &str) -> Result<Self, SwarmRefusal> {
        if !is_stable_id(cluster_id) {
            return Err(SwarmRefusal::InvalidClusterId);
        }
        Ok(Self {
            cluster_id: cluster_id.to_string(),
            swarms: Vec::new(),
            links: Vec::new(),
        })
    }

    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub fn swarms(&self) -> &[SwarmDescriptor] {
        &self.swarms
    }

    pub fn links(&self) -> &[SwarmLink] {
        &self.links
    }

    /// Admit one swarm descriptor. This validates identifiers and references;
    /// it does not connect to the endpoint or resolve the authority reference.
    pub fn admit_swarm(
        &mut self,
        id: &str,
        gateway_node: &str,
        authority_ref: &str,
        endpoint: &str,
    ) -> Result<(), SwarmRefusal> {
        if !is_stable_id(id) {
            return Err(SwarmRefusal::InvalidSwarmId);
        }
        if !is_stable_id(gateway_node) {
            return Err(SwarmRefusal::InvalidGatewayNode);
        }
        if authority_ref.is_empty() {
            return Err(SwarmRefusal::MissingAuthorityRef);
        }
        if !endpoint.starts_with("atomvm://") || endpoint.len() == "atomvm://".len() {
            return Err(SwarmRefusal::InvalidEndpoint);
        }
        if self.swarms.iter().any(|swarm| swarm.id == id) {
            return Err(SwarmRefusal::DuplicateSwarm);
        }

        self.swarms.push(SwarmDescriptor {
            id: id.to_string(),
            gateway_node: gateway_node.to_string(),
            authority_ref: authority_ref.to_string(),
            endpoint: endpoint.to_string(),
        });
        self.swarms.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(())
    }

    /// Add an undirected topology edge. Duplicate links are idempotent, matching
    /// the Set semantics in the unrdf source.
    pub fn connect(&mut self, left: &str, right: &str) -> Result<(), SwarmRefusal> {
        if left == right {
            return Err(SwarmRefusal::SelfLink);
        }
        self.require_swarm(left)?;
        self.require_swarm(right)?;

        let (canonical_left, canonical_right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };

        if self
            .links
            .iter()
            .any(|link| link.left == canonical_left && link.right == canonical_right)
        {
            return Ok(());
        }

        self.links.push(SwarmLink {
            left: canonical_left.to_string(),
            right: canonical_right.to_string(),
        });
        self.links
            .sort_by(|a, b| a.left.cmp(&b.left).then(a.right.cmp(&b.right)));
        Ok(())
    }

    /// Select a deterministic shortest path with lexicographically ordered BFS.
    /// The returned route is information only; no network call is performed.
    pub fn route(&self, source: &str, target: &str) -> Result<Vec<String>, SwarmRefusal> {
        self.require_swarm(source)?;
        self.require_swarm(target)?;

        if source == target {
            return Ok(alloc::vec![source.to_string()]);
        }

        let mut queue = VecDeque::new();
        queue.push_back(alloc::vec![source.to_string()]);
        let mut visited = alloc::vec![source.to_string()];

        while let Some(path) = queue.pop_front() {
            let current = match path.last() {
                Some(value) => value,
                None => continue,
            };

            for next in self.neighbors(current) {
                if visited.iter().any(|seen| seen == &next) {
                    continue;
                }
                visited.push(next.clone());
                let mut candidate = path.clone();
                candidate.push(next.clone());
                if next == target {
                    return Ok(candidate);
                }
                queue.push_back(candidate);
            }
        }

        Err(SwarmRefusal::NoRoute)
    }

    fn require_swarm(&self, id: &str) -> Result<&SwarmDescriptor, SwarmRefusal> {
        self.swarms
            .iter()
            .find(|swarm| swarm.id == id)
            .ok_or(SwarmRefusal::UnknownSwarm)
    }

    fn neighbors(&self, id: &str) -> Vec<String> {
        let mut neighbors = Vec::new();
        for link in &self.links {
            if link.left == id {
                neighbors.push(link.right.clone());
            } else if link.right == id {
                neighbors.push(link.left.clone());
            }
        }
        neighbors.sort();
        neighbors.dedup();
        neighbors
    }
}

/// Source-compatible stable-id grammar without a regex dependency.
pub fn is_stable_id(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topology() -> SwarmTopology {
        let mut topology = SwarmTopology::new("cluster-1").unwrap();
        for id in ["a", "b", "c", "d"] {
            topology
                .admit_swarm(id, id, "cookie-ref", &alloc::format!("atomvm://{}", id))
                .unwrap();
        }
        topology
    }

    #[test]
    fn validates_admission_without_resolving_external_authority() {
        assert!(SwarmTopology::new("bad id").is_err());
        let mut topology = SwarmTopology::new("c1").unwrap();
        assert_eq!(
            topology.admit_swarm("s1", "node1", "", "atomvm://s1"),
            Err(SwarmRefusal::MissingAuthorityRef)
        );
        assert_eq!(
            topology.admit_swarm("s1", "node1", "ref", "https://s1"),
            Err(SwarmRefusal::InvalidEndpoint)
        );
        topology
            .admit_swarm("s1", "node1", "ref", "atomvm://s1")
            .unwrap();
        assert_eq!(
            topology.admit_swarm("s1", "node1", "ref", "atomvm://s1"),
            Err(SwarmRefusal::DuplicateSwarm)
        );
    }

    #[test]
    fn bfs_route_is_shortest_and_deterministic() {
        let mut topology = topology();
        // Two equal-length routes exist; lexical neighbor order selects b.
        topology.connect("a", "c").unwrap();
        topology.connect("c", "d").unwrap();
        topology.connect("a", "b").unwrap();
        topology.connect("b", "d").unwrap();
        assert_eq!(
            topology.route("a", "d").unwrap(),
            alloc::vec!["a".to_string(), "b".to_string(), "d".to_string()]
        );
    }

    #[test]
    fn links_are_idempotent_and_unreachable_routes_refuse() {
        let mut topology = topology();
        topology.connect("a", "b").unwrap();
        topology.connect("b", "a").unwrap();
        assert_eq!(topology.links().len(), 1);
        assert_eq!(topology.route("a", "d"), Err(SwarmRefusal::NoRoute));
        assert_eq!(topology.connect("a", "a"), Err(SwarmRefusal::SelfLink));
    }
}
