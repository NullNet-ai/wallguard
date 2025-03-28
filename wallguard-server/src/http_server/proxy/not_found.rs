pub const NOT_FOUND_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 Not Found</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            background-color: #121212;
            color: #ffffff;
            text-align: center;
            padding: 50px;
        }
        h1 {
            font-size: 72px;
            margin: 20px 0;
        }
        p {
            font-size: 24px;
        }
    </style>
</head>
<body>
    <h1>404</h1>
    <p>The session you're requesting either does not exist or not online yet.</p>
</body>
</html>"#;
