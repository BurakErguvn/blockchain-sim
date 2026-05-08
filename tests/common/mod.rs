use blockchain_sim::network::BlockchainNetwork;

pub fn setup_network(node_count: usize) -> BlockchainNetwork {
    let mut network = BlockchainNetwork::new();
    network.set_difficulty(1);
    network.set_block_time(1);

    for _ in 0..node_count {
        network.add_node();
    }

    for i in 0..network.node_count() {
        for j in (i + 1)..network.node_count() {
            network.connect_nodes(i, j);
        }
    }

    network
}

pub fn mine_genesis(network: &mut BlockchainNetwork) {
    network.select_random_validator();
    let mined = network.mine_block();
    assert!(mined.is_some(), "Genesis bloğu oluşturulmalı");
}

pub fn find_funded_node(network: &BlockchainNetwork) -> Option<usize> {
    network
        .nodes
        .iter()
        .position(|node| node.wallet.get_balance() > 0)
}
