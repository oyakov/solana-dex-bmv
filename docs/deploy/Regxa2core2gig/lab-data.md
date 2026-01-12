# Lab Data

Login with ssh key
ssh root@146.103.42.174

folder /opt/solana-dex-bmv

## Security Hardening (Recommendations)

To secure this lab environment, follow these steps:

1.  **Restrict SSH Access**:
    - Edit `/etc/ssh/sshd_config` and set `PermitRootLogin prohibit-password` (or `no` if you create a sudo user).
    - Disable password authentication: `PasswordAuthentication no`.
    - Restart SSH: `systemctl restart ssh`.

2.  **Enable Firewall (UFW)**:
    ```bash
    apt update && apt install ufw -y
    ufw default deny incoming
    ufw default allow outgoing
    ufw allow ssh
    # Allow local access to bot/observability if needed via VPN/Tunnel
    # ufw allow from YOUR_IP to any port 3000
    ufw enable
    ```

3.  **Update System**:
    ```bash
    apt update && apt upgrade -y
    ```

4.  **Secrets Management**:
    - Do **NOT** put private keys in `config.yaml`.
    - Use a `.env` file on the server and set `WALLET_KEYPAIRS="your,keys,here"`.
    - Ensure `.env` is `chmod 600`.

