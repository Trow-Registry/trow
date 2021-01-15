import React from "react";
import { Button, Segment } from "semantic-ui-react";
import { Link } from "react-router-dom";

export default function Home() {
    return (
        <Segment textAlign="center" basic>
            <h2>Trow</h2>
            <h4>The Cloud Native Registry</h4>
            <Button basic as={Link} to="/login">
                Login
            </Button>
            <Button basic as={Link} to="/repositories">
                Repositories
            </Button>
        </Segment>
    );
}
