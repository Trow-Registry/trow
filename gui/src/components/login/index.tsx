import React, { useState } from "react";
import { Header, Form, Button, Segment } from "semantic-ui-react";

import styled from "styled-components";

const LoginSegment = styled(Segment)`
    padding: 40% 20% 0 20% !important;
`;

const Login = () => {
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");

    // WIP
    const handleSubmit = () => {
        console.log(username);
        console.log(password);
    };

    return (
        <LoginSegment basic>
            <Form onSubmit={handleSubmit}>
                <Header>Log in</Header>
                <Form.Input
                    label="Username"
                    placeholder="Username"
                    id="form-input-username"
                    // width="4"
                    required
                    onChange={(e) => setUsername(e.target.value)}
                />
                <Form.Input
                    label="Password"
                    placeholder="Password"
                    id="form-input-password"
                    // width="4"
                    type="password"
                    required
                    onChange={(e) => setPassword(e.target.value)}
                />
                <Button basic type="submit">
                    Login
                </Button>
            </Form>
        </LoginSegment>
    );
};

export default Login;
