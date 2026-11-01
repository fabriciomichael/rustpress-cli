# RustPress CLI

**A ferramenta oficial de linha de comandos do framework RustPress.**

O RustPress CLI é uma ferramenta desenvolvida em Rust que permite criar automaticamente a estrutura inicial de aplicações web utilizando o framework [RustPress](https://github.com/fabriciomichael/rustpress-web).

O seu objectivo é simplificar o processo de criação de projectos, evitando a configuração manual inicial de directórios e ficheiros.

## Funcionalidades

- Criação automática de projectos RustPress.
- Geração da estrutura de directórios.
- Criação dos ficheiros iniciais da aplicação.
- Configuração das dependências do projecto.
- Organização de rotas e controladores.
- Estrutura inicial para modelos, middlewares e configurações.
- Validação dos nomes dos projectos.
- Protecção contra nomes reservados do Windows.

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install)
- Cargo
- Framework RustPress (`rustpress-web`)

## Instalação

Após a publicação no crates.io, instala o RustPress CLI através do Cargo:

```bash
cargo install rustpress-cli
```

Depois da instalação, o comando `rustpress` estará disponível no terminal.

## Utilização

### Criar um projecto

```bash
rustpress new minha_app
```

### Entrar no projecto

```bash
cd minha_app
```

### Executar a aplicação

```bash
cargo run
```

Por padrão, o servidor é configurado para iniciar em:

```text
http://127.0.0.1:3000
```

### Consultar os comandos disponíveis

```bash
rustpress --help
```

### Consultar a versão

```bash
rustpress --version
```

## Estrutura do projecto

O RustPress CLI gera uma estrutura inicial semelhante à seguinte:

```text
minha_app/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── web.rs
│   ├── controllers/
│   │   ├── mod.rs
│   │   └── home_controller.rs
│   ├── models/
│   │   └── mod.rs
│   ├── middleware/
│   │   └── mod.rs
│   └── config/
│       └── mod.rs
├── views/
│   ├── layouts/
│   ├── pages/
│   │   └── home.html
│   └── partials/
├── public/
│   ├── css/
│   ├── js/
│   └── images/
└── tests/
```

## Tecnologias

- **Rust:** linguagem de programação.
- **Clap:** biblioteca para processamento de argumentos e comandos CLI.
- **RustPress Web:** framework utilizado pelas aplicações geradas.

## Desenvolvimento

Para obter o código-fonte:

```bash
git clone https://github.com/fabriciomichael/rustpress-cli.git
```

Entrar no directório:

```bash
cd rustpress-cli
```

Executar os testes:

```bash
cargo test
```

Executar o CLI durante o desenvolvimento:

```bash
cargo run -- new minha_app
```

Instalar a versão local:

```bash
cargo install --path . --force
```

## Projectos relacionados

- [RustPress Web](https://github.com/fabriciomichael/rustpress-web)

## Licença

Este projecto é distribuído sob a licença MIT. Consulta o ficheiro [LICENSE](LICENSE) para mais informações.

## Documentação

- [Configurar o main.rs](docs/CONFIGURAR-MAIN.md): guia para configurar e iniciar uma aplicação utilizando o RustPress Web.