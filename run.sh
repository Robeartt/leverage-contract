#!/bin/bash

# Function to connect to leverage-contract container
connect_to_container() {
  echo "Connecting to leverage-contract container..."
  docker exec --tty --interactive leverage-contract bash
}

if [[ $# -eq 0 ]]; then
  # No arguments, connect to leverage-contract
  connect_to_container
elif [[ $1 == "--no-blockchain" || $1 == "--nb" ]]; then
  # With --no-blockchain, start only leverage-contract container and connect
  echo "Starting only leverage-contract container..."
  docker-compose up -d leverage-contract
  connect_to_container
else
  # Any other argument, just connect to leverage-contract
  connect_to_container
fi
