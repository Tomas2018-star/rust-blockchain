mod common;

use common::{Api, ServerBuilder};
use serial_test::serial;

#[test]
#[serial]
#[cfg(windows)]
fn test_should_receive_new_valid_blocks() {
    let leader_node = ServerBuilder::new().port(8000).start();
    let mut follower_node = ServerBuilder::new().port(8001).peer(8000).start();

    assert_eq!(leader_node.get_blocks().len(), 1);
    assert_eq!(follower_node.get_blocks().len(), 1);

    leader_node.add_valid_block();
    assert_eq!(leader_node.get_blocks().len(), 2);

    follower_node.wait_for_peer_sync();
    assert_eq!(follower_node.get_blocks().len(), 2);

    let last_leader_block = leader_node.get_last_block();
    let last_follower_block = follower_node.get_last_block();
    assert_eq!(last_follower_block, last_leader_block);
}

#[test]
#[serial]
#[cfg(windows)]
fn test_should_not_receive_new_invalid_blocks() {
    let leader_node = ServerBuilder::new().port(8000).start();
    // different difficulty from leader node, it should not accept blocks from leader node
    let mut follower_node = ServerBuilder::new()
        .difficulty(20)
        .port(8001)
        .peer(8000)
        .start();

    leader_node.add_valid_block();
    follower_node.wait_for_peer_sync();

    assert_eq!(follower_node.get_blocks().len(), 1);
}

#[test]
#[serial]
#[cfg(windows)]
fn test_should_send_new_blocks() {
    let mut follower_node = ServerBuilder::new().port(8000).start();
    let leader_node = ServerBuilder::new().port(8001).peer(8000).start();

    assert_eq!(leader_node.get_blocks().len(), 1);
    assert_eq!(follower_node.get_blocks().len(), 1);

    leader_node.add_valid_block();
    assert_eq!(leader_node.get_blocks().len(), 2);

    follower_node.wait_to_receive_block_in_api();
    assert_eq!(follower_node.get_blocks().len(), 2);

    let last_leader_block = leader_node.get_last_block();
    let last_follower_block = follower_node.get_last_block();
    assert_eq!(last_follower_block, last_leader_block);
}
