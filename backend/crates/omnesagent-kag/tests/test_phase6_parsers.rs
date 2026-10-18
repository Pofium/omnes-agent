//! Комплексные юнит-тесты AST-экстрактора для форматов и языков Фазы 6:
//! - HTML & CSS/SCSS
//! - Protobuf & GraphQL & OpenAPI
//! - Terraform & Dockerfile & Docker Compose & Kubernetes
//! - Пакетные манифесты (Cargo, NPM, Python, Go, Dart, Composer)
//! - Jupyter Notebooks, Solidity, Zig

use omnesagent_kag::ast::{AstCodeExtractor, AstScanResult};

#[test]
fn test_parse_html() {
    let extractor = AstCodeExtractor::new();
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" href="assets/styles/main.css">
    <script src="assets/js/bundle.js" defer></script>
</head>
<body>
    <header id="main-header" class="header sticky-nav">
        <app-navbar></app-navbar>
    </header>
    <main id="content" class="container page-content">
        <section class="hero-banner">
            <h1>OmnesAgent</h1>
        </section>
    </main>
</body>
</html>"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("public/index.html", html, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"#main-header".to_string()));
    assert!(node_labels.contains(&"#content".to_string()));
    assert!(node_labels.contains(&"<app-navbar>".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"IMPORTS".to_string()));
    assert!(edge_labels.contains(&"STYLES".to_string()));
    assert!(edge_labels.contains(&"DEFINES".to_string()));
    assert!(edge_labels.contains(&"BINDS_CLASS".to_string()));
}

#[test]
fn test_parse_css_scss() {
    let extractor = AstCodeExtractor::new();
    let scss = r#"
@import "base/variables.scss";

:root {
    --primary-color: #00ffcc;
    --bg-dark: #0f172a;
}

@mixin flex-center {
    display: flex;
    justify-content: center;
    align-items: center;
}

.hero-card {
    background-color: var(--bg-dark);
    color: var(--primary-color);
}

#login-form {
    padding: 2rem;
}

@keyframes pulse {
    0% { transform: scale(1); }
    50% { transform: scale(1.05); }
    100% { transform: scale(1); }
}
"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("styles/theme.scss", scss, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"--primary-color".to_string()));
    assert!(node_labels.contains(&"@mixin flex-center".to_string()));
    assert!(node_labels.contains(&".hero-card".to_string()));
    assert!(node_labels.contains(&"#login-form".to_string()));
    assert!(node_labels.contains(&"@keyframes pulse".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"IMPORTS".to_string()));
    assert!(edge_labels.contains(&"DEFINES".to_string()));
}

#[test]
fn test_parse_protobuf() {
    let extractor = AstCodeExtractor::new();
    let proto = r#"syntax = "proto3";

package omnes.agent.v1;

import "google/protobuf/timestamp.proto";

enum AgentStatus {
    UNKNOWN = 0;
    IDLE = 1;
    BUSY = 2;
}

message RunTaskRequest {
    string task_id = 1;
    string prompt = 2;
}

message RunTaskResponse {
    string task_id = 1;
    AgentStatus status = 2;
}

service TaskAgentService {
    rpc RunTask (RunTaskRequest) returns (RunTaskResponse);
}
"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("proto/agent.proto", proto, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"package omnes.agent.v1".to_string()));
    assert!(node_labels.contains(&"AgentStatus".to_string()));
    assert!(node_labels.contains(&"RunTaskRequest".to_string()));
    assert!(node_labels.contains(&"RunTaskResponse".to_string()));
    assert!(node_labels.contains(&"TaskAgentService".to_string()));
    assert!(node_labels.contains(&"RunTask".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"IMPORTS".to_string()));
    assert!(edge_labels.contains(&"DEFINES".to_string()));
    assert!(edge_labels.contains(&"DEPENDS_ON".to_string()));
}

#[test]
fn test_parse_graphql() {
    let extractor = AstCodeExtractor::new();
    let gql = r#"
interface Node {
    id: ID!
}

enum Role {
    ADMIN
    USER
    GUEST
}

type User implements Node {
    id: ID!
    username: String!
    role: Role!
}

input CreateUserInput {
    username: String!
    role: Role
}

type Query {
    getUser(id: ID!): User
}

type Mutation {
    createUser(input: CreateUserInput!): User
}
"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("schema.graphql", gql, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"Node".to_string()));
    assert!(node_labels.contains(&"Role".to_string()));
    assert!(node_labels.contains(&"User".to_string()));
    assert!(node_labels.contains(&"CreateUserInput".to_string()));
    assert!(node_labels.contains(&"Query.getUser".to_string()));
    assert!(node_labels.contains(&"Mutation.createUser".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"DEFINES".to_string()));
    assert!(edge_labels.contains(&"IMPLEMENTS".to_string()));
}

#[test]
fn test_parse_openapi() {
    let extractor = AstCodeExtractor::new();
    let openapi_json = r#"{
  "openapi": "3.0.3",
  "info": {
    "title": "Omnes Gateway API",
    "version": "1.0.0"
  },
  "paths": {
    "/api/sessions": {
      "get": {
        "operationId": "listSessions",
        "summary": "List all active sessions"
      },
      "post": {
        "operationId": "createSession",
        "summary": "Spawn a new agent session"
      }
    }
  },
  "components": {
    "schemas": {
      "SessionDto": {
        "type": "object"
      },
      "CreateSessionRequest": {
        "type": "object"
      }
    }
  }
}"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("api/openapi.json", openapi_json, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"Omnes Gateway API".to_string()));
    assert!(node_labels.contains(&"GET /api/sessions".to_string()));
    assert!(node_labels.contains(&"POST /api/sessions".to_string()));
    assert!(node_labels.contains(&"SessionDto".to_string()));
    assert!(node_labels.contains(&"CreateSessionRequest".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"DEFINES".to_string()));
    assert!(edge_labels.contains(&"EXPOSES".to_string()));
}

#[test]
fn test_parse_terraform() {
    let extractor = AstCodeExtractor::new();
    let tf = r#"
terraform {
  required_version = ">= 1.5.0"
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  type    = string
  default = "us-east-1"
}

data "aws_ami" "ubuntu" {
  most_recent = true
}

module "vpc" {
  source = "terraform-aws-modules/vpc/aws"
}

resource "aws_instance" "app_server" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = "t3.micro"
}

output "instance_ip" {
  value = aws_instance.app_server.public_ip
}
"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("infra/main.tf", tf, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"provider.aws".to_string()));
    assert!(node_labels.contains(&"var.aws_region".to_string()));
    assert!(node_labels.contains(&"data.aws_ami.ubuntu".to_string()));
    assert!(node_labels.contains(&"module.vpc".to_string()));
    assert!(node_labels.contains(&"aws_instance.app_server".to_string()));
    assert!(node_labels.contains(&"output.instance_ip".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"DEFINES".to_string()));
    assert!(edge_labels.contains(&"DEPENDS_ON".to_string()));
    assert!(edge_labels.contains(&"IMPORTS".to_string()));
}

#[test]
fn test_parse_docker_and_compose() {
    let extractor = AstCodeExtractor::new();
    let dockerfile = r#"FROM rust:1.80-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.20
COPY --from=builder /app/target/release/server /usr/local/bin/server
EXPOSE 8080
CMD ["server"]
"#;

    let mut res_df = AstScanResult::default();
    extractor.parse_file("Dockerfile", dockerfile, &mut res_df);

    let df_labels: Vec<String> = res_df.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(df_labels.iter().any(|l| l.contains("FROM rust:1.80-alpine AS builder")));
    assert!(df_labels.iter().any(|l| l.contains("FROM alpine:3.20")));
    assert!(df_labels.contains(&"EXPOSE 8080".to_string()));

    let compose = r#"services:
  gateway:
    image: omnesagent/gateway:latest
    ports:
      - "8080:8080"
    depends_on:
      - redis
      - db
  redis:
    image: redis:7-alpine
  db:
    image: postgres:16-alpine
"#;

    let mut res_comp = AstScanResult::default();
    extractor.parse_file("docker-compose.yml", compose, &mut res_comp);

    let comp_labels: Vec<String> = res_comp.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(comp_labels.contains(&"gateway".to_string()));
    assert!(comp_labels.contains(&"redis".to_string()));
    assert!(comp_labels.contains(&"db".to_string()));

    let comp_edges: Vec<String> = res_comp.edges.iter().map(|e| e.label.clone()).collect();
    assert!(comp_edges.contains(&"DEFINES".to_string()));
    assert!(comp_edges.contains(&"DEPENDS_ON".to_string()));
}

#[test]
fn test_parse_kubernetes() {
    let extractor = AstCodeExtractor::new();
    let k8s_yaml = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: omnes-gateway
spec:
  replicas: 3
---
apiVersion: v1
kind: Service
metadata:
  name: omnes-gateway-svc
spec:
  type: ClusterIP
"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("deploy/k8s.yaml", k8s_yaml, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"Deployment:omnes-gateway".to_string()));
    assert!(node_labels.contains(&"Service:omnes-gateway-svc".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"DEFINES".to_string()));
}

#[test]
fn test_parse_package_manifests() {
    let extractor = AstCodeExtractor::new();

    // 1. Cargo.toml
    let cargo = r#"[package]
name = "omnes-micro"
version = "0.1.0"

[dependencies]
tokio = "1.0"
serde = { version = "1.0", features = ["derive"] }
"#;
    let mut res_cargo = AstScanResult::default();
    extractor.parse_file("Cargo.toml", cargo, &mut res_cargo);
    let cargo_labels: Vec<String> = res_cargo.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(cargo_labels.contains(&"omnes-micro".to_string()));
    assert!(cargo_labels.contains(&"tokio".to_string()));
    assert!(cargo_labels.contains(&"serde".to_string()));

    // 2. package.json
    let npm = r#"{
  "name": "omnes-web-client",
  "dependencies": {
    "react": "^18.2.0",
    "axios": "^1.6.0"
  }
}"#;
    let mut res_npm = AstScanResult::default();
    extractor.parse_file("package.json", npm, &mut res_npm);
    let npm_labels: Vec<String> = res_npm.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(npm_labels.contains(&"omnes-web-client".to_string()));
    assert!(npm_labels.contains(&"react".to_string()));
    assert!(npm_labels.contains(&"axios".to_string()));

    // 3. go.mod
    let go_mod = r#"module github.com/omnes/backend

go 1.22

require (
    github.com/gin-gonic/gin v1.9.1
    github.com/stretchr/testify v1.8.4
)
"#;
    let mut res_go = AstScanResult::default();
    extractor.parse_file("go.mod", go_mod, &mut res_go);
    let go_labels: Vec<String> = res_go.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(go_labels.contains(&"github.com/omnes/backend".to_string()));
    assert!(go_labels.contains(&"github.com/gin-gonic/gin".to_string()));

    // 4. requirements.txt
    let reqs = r#"
fastapi>=0.100.0
uvicorn==0.23.0
pydantic
"#;
    let mut res_py = AstScanResult::default();
    extractor.parse_file("requirements.txt", reqs, &mut res_py);
    let py_labels: Vec<String> = res_py.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(py_labels.contains(&"fastapi".to_string()));
    assert!(py_labels.contains(&"uvicorn".to_string()));
    assert!(py_labels.contains(&"pydantic".to_string()));
}

#[test]
fn test_parse_jupyter_notebook() {
    let extractor = AstCodeExtractor::new();
    let ipynb = r##"{
  "cells": [
    {
      "cell_type": "markdown",
      "source": ["# Analysis Notebook\n"]
    },
    {
      "cell_type": "code",
      "source": [
        "import numpy as np\n",
        "import pandas as pd\n",
        "\n",
        "class ModelEvaluator:\n",
        "    def evaluate(self, df):\n",
        "        return df.describe()\n"
      ]
    }
  ],
  "metadata": {},
  "nbformat": 4,
  "nbformat_minor": 2
}"##;

    let mut res = AstScanResult::default();
    extractor.parse_file("notebooks/eval.ipynb", ipynb, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"Cell [1]".to_string()));
    assert!(node_labels.contains(&"ModelEvaluator".to_string()));
    assert!(node_labels.contains(&"evaluate".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"DEFINES".to_string()));
    assert!(edge_labels.contains(&"IMPORTS".to_string()));
}

#[test]
fn test_parse_solidity() {
    let extractor = AstCodeExtractor::new();
    let sol = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract OmnesToken is ERC20, Ownable {
    event Minted(address indexed to, uint256 amount);

    modifier onlyValidAddress(address account) {
        require(account != address(0), "Invalid address");
        _;
    }

    constructor() ERC20("OmnesToken", "OMN") Ownable(msg.sender) {}

    function mint(address to, uint256 amount) public onlyOwner onlyValidAddress(to) {
        _mint(to, amount);
        emit Minted(to, amount);
    }
}
"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("contracts/OmnesToken.sol", sol, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"OmnesToken".to_string()));
    assert!(node_labels.contains(&"Minted".to_string()));
    assert!(node_labels.contains(&"onlyValidAddress".to_string()));
    assert!(node_labels.contains(&"mint".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"IMPORTS".to_string()));
    assert!(edge_labels.contains(&"DEFINES".to_string()));
    assert!(edge_labels.contains(&"INHERITS".to_string()));
}

#[test]
fn test_parse_zig() {
    let extractor = AstCodeExtractor::new();
    let zig = r#"
const std = @import("std");

pub const Point = struct {
    x: f32,
    y: f32,
};

pub const Status = enum {
    Active,
    Inactive,
};

pub fn calculateDistance(p1: Point, p2: Point) f32 {
    const dx = p1.x - p2.x;
    const dy = p1.y - p2.y;
    return @sqrt(dx * dx + dy * dy);
}

pub fn main() void {
    std.debug.print("Hello Zig!\n", .{});
}
"#;

    let mut res = AstScanResult::default();
    extractor.parse_file("src/main.zig", zig, &mut res);

    let node_labels: Vec<String> = res.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(node_labels.contains(&"Point".to_string()));
    assert!(node_labels.contains(&"Status".to_string()));
    assert!(node_labels.contains(&"calculateDistance".to_string()));
    assert!(node_labels.contains(&"main".to_string()));

    let edge_labels: Vec<String> = res.edges.iter().map(|e| e.label.clone()).collect();
    assert!(edge_labels.contains(&"IMPORTS".to_string()));
    assert!(edge_labels.contains(&"DEFINES".to_string()));
}
