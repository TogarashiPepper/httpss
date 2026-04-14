#!/usr/bin/env bash
set -euo pipefail

openssl req -x509 -nodes -newkey rsa:2048 -keyout ca.key -out ca.crt -subj "/CN=MyLocalCA"
openssl req -nodes -newkey rsa:2048 -keyout server.key -out server.csr -subj "/CN=localhost"
cat << EOF > ext.conf
basicConstraints = CA:FALSE
subjectAltName = DNS:localhost
EOF
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 365 -sha256 -extfile ext.conf
rm server.csr ext.conf ca.srl
