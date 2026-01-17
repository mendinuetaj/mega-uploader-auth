use crate::db;
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};

#[get("/")]
pub async fn info(redis_pool: web::Data<db::RedisPool>) -> impl Responder {
    info!("Processing request on root path");

    let html_content = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Mega Uploader Auth - API Info</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            body {
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                display: flex;
                justify-content: center;
                align-items: center;
                padding: 20px;
            }
            .container {
                background: white;
                border-radius: 20px;
                box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                max-width: 800px;
                width: 100%;
                overflow: hidden;
                animation: slideIn 0.5s ease-out;
            }
            @keyframes slideIn {
                from {
                    opacity: 0;
                    transform: translateY(-30px);
                }
                to {
                    opacity: 1;
                    transform: translateY(0);
                }
            }
            .header {
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                padding: 40px;
                text-align: center;
            }
            .header h1 {
                font-size: 2.5em;
                margin-bottom: 10px;
            }
            .header p {
                opacity: 0.9;
                font-size: 1.1em;
            }
            .content {
                padding: 40px;
            }
            .info-section {
                margin-bottom: 30px;
            }
            .info-section h2 {
                color: #667eea;
                margin-bottom: 15px;
                font-size: 1.5em;
                border-left: 4px solid #667eea;
                padding-left: 15px;
            }
            .api-card {
                background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
                border-radius: 10px;
                padding: 20px;
                margin-bottom: 15px;
                transition: transform 0.3s ease, box-shadow 0.3s ease;
            }
            .api-card:hover {
                transform: translateY(-5px);
                box-shadow: 0 10px 25px rgba(0,0,0,0.1);
            }
            .endpoint {
                font-family: 'Courier New', monospace;
                background: #667eea;
                color: white;
                padding: 8px 15px;
                border-radius: 5px;
                display: inline-block;
                font-weight: bold;
                margin-bottom: 10px;
            }
            .description {
                color: #333;
                line-height: 1.6;
            }
            .badge {
                display: inline-block;
                padding: 5px 15px;
                border-radius: 20px;
                font-size: 0.9em;
                margin: 5px;
            }
            .badge-version {
                background: #48bb78;
                color: white;
            }
            .badge-db {
                background: #ed8936;
                color: white;
            }
            .footer {
                background: #f7fafc;
                padding: 20px;
                text-align: center;
                color: #718096;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1>🚀 Mega Uploader Auth</h1>
                <p>Authentication & API Management System</p>
                <div style="margin-top: 20px;">
                    <span class="badge badge-version">v0.1.0</span>
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
                <p>Built with ❤️ using Rust</p>
            </div>
        </div>
    </body>
    </html>
    "#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_content)
}
