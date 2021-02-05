import React, { useState } from "react";
import { Header, Form, Button, Segment } from "semantic-ui-react";

const Login = () => {
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");

    // WIP
    const handleSubmit = () => {
        console.log(username);
        console.log(password);
    };

    return (
        <Segment basic>
            <Form onSubmit={handleSubmit}>
                <Header>Log in</Header>
                <Form.Input
                    label="Username"
                    placeholder="Username"
                    id="form-input-username"
                    width="4"
                    required
                    onChange={(e) => setUsername(e.target.value)}
                />
                <Form.Input
                    label="Password"
                    placeholder="Password"
                    id="form-input-password"
                    width="4"
                    type="password"
                    required
                    onChange={(e) => setPassword(e.target.value)}
                />
                <Button basic type="submit">
                    Login
                </Button>
            </Form>
        </Segment>
    );
};

export default Login;
