import React from "react";
import { Menu, Icon, Header } from "semantic-ui-react";
import { Link } from "react-router-dom";

export default function MainHeader() {
    return (
        // <Segment basic clearing>

        <Menu borderless size="mini">
            <Menu.Item>
                <Header floated="left" as={Link} to="/">
                    Trow
                </Header>
            </Menu.Item>

            <Menu.Menu position="right">
                {/* <Menu.Item>
                        <Input
                            action={{ type: 'submit', content: 'Search images' }}
                            placeholder='Search images...'
                        />
                    </Menu.Item> */}
                <Menu.Item>
                    <Icon name="user outline"></Icon>
                </Menu.Item>
            </Menu.Menu>
        </Menu>

        // </Segment>
    );
}
