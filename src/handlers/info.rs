use actix_web::{get, HttpResponse, Responder};
use log::info;

/// Handler for the root path that returns information about the API.
///
/// It renders a simple HTML page with the project name, version, and available endpoints.
#[get("/")]
pub async fn info() -> impl Responder {
    info!("Processing request on root path");

    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

    let html_content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{} - API Info</title>
        <style>
            * {{
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }}
            body {{
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                display: flex;
                justify-content: center;
                align-items: center;
                padding: 20px;
            }}
            .container {{
                background: white;
                border-radius: 20px;
                box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                max-width: 800px;
                width: 100%;
                overflow: hidden;
                animation: slideIn 0.5s ease-out;
            }}
            @keyframes slideIn {{
                from {{
                    opacity: 0;
                    transform: translateY(-30px);
                }}
                to {{
                    opacity: 1;
                    transform: translateY(0);
                }}
            }}
            .header {{
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                padding: 40px;
                text-align: center;
            }}
            .header h1 {{
                font-size: 2.5em;
                margin-bottom: 10px;
            }}
            .header p {{
                opacity: 0.9;
                font-size: 1.1em;
            }}
            .content {{
                padding: 40px;
            }}
            .info-section {{
                margin-bottom: 30px;
            }}
            .info-section h2 {{
                color: #667eea;
                margin-bottom: 15px;
                font-size: 1.5em;
                border-left: 4px solid #667eea;
                padding-left: 15px;
            }}
            .api-card {{
                background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
                border-radius: 10px;
                padding: 20px;
                margin-bottom: 15px;
                transition: transform 0.3s ease, box-shadow 0.3s ease;
            }}
            .api-card:hover {{
                transform: translateY(-5px);
                box-shadow: 0 10px 25px rgba(0,0,0,0.1);
            }}
            .endpoint {{
                font-family: 'Courier New', monospace;
                background: #667eea;
                color: white;
                padding: 8px 15px;
                border-radius: 5px;
                display: inline-block;
                font-weight: bold;
                margin-bottom: 10px;
            }}
            .description {{
                color: #333;
                line-height: 1.6;
            }}
            .badge {{
                display: inline-block;
                padding: 5px 15px;
                border-radius: 20px;
                font-size: 0.9em;
                margin: 5px;
            }}
            .badge-version {{
                background: #48bb78;
                color: white;
            }}
            .badge-db {{
                background: #ed8936;
                color: white;
            }}
            .footer {{
                background: #f7fafc;
                padding: 20px;
                text-align: center;
                color: #718096;
            }}
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1>🚀 {}</h1>
                <p>{}</p>
                <div style="margin-top: 20px;">
                    <span class="badge badge-version">v{}</span>
                    <span class="badge badge-db">Redis + BB8</span>
                </div>
            </div>

            <div class="content">
                <div class="info-section">
                    <h2>📡 Available APIs</h2>

                    <div class="api-card">
                        <div class="endpoint">GET /</div>
                        <div class="description">
                            Returns detailed information about the available APIs in the project.
                            Main interactive documentation endpoint.
                        </div>
                    </div>

                    <div class="api-card">
                        <div class="endpoint">POST /auth/cli/start</div>
                        <div class="description">
                            Initiates the CLI authentication process. Returns an authorization URL for the browser.
                        </div>
                    </div>

                    <div class="api-card">
                        <div class="endpoint">GET /auth/cli/callback</div>
                        <div class="description">
                            Callback endpoint for the identity provider. Handles token exchange and session creation.
                        </div>
                    </div>

                    <div class="api-card">
                        <div class="endpoint">GET /auth/cli/status</div>
                        <div class="description">
                            Polls the authentication status for a specific state. Returns AWS STS credentials if authorized.
                        </div>
                    </div>

                    <div class="api-card">
                        <div class="endpoint">POST /auth/cli/renew</div>
                        <div class="description">
                            Renews an expired CLI session using a refresh token to obtain new AWS STS credentials.
                        </div>
                    </div>
                </div>

                <div class="info-section">
                    <h2>🔧 Technologies</h2>
                    <div class="api-card">
                        <div class="description">
                            <strong>Framework:</strong> Actix Web 4<br>
                            <strong>Database:</strong> Redis with BB8 connection pool<br>
                            <strong>Logging:</strong> env_logger + log<br>
                            <strong>Serialization:</strong> Serde
                        </div>
                    </div>
                </div>
            </div>

            <div class="footer">
                <p>Powered by DPAAS | <a href="https://dpaasglint.com/" target="_blank" style="color: #667eea; text-decoration: none;">DPAAS</a></p>
            </div>
        </div>
    </body>
    </html>
    "#,
        PKG_NAME, PKG_NAME, PKG_DESCRIPTION, PKG_VERSION
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_content)
}
