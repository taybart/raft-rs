#[derive(Default, PartialEq, Clone)]
pub enum Role {
    #[default]
    Follower,
    Candidate,
    Leader,
}

// impl Role {
//     pub fn from_string(role: String) -> Result<Role, String> {
//         match role.as_str() {
//             "follower" => Ok(Self::Follower),
//             "candidate" => Ok(Self::Candidate),
//             "leader" => Ok(Self::Leader),
//             _ => Err("unknown role".to_string()),
//         }
//     }
// }
