import React, { memo } from "react";
import { Menu, Header } from "semantic-ui-react";
import { Link } from "react-router-dom";

export function MainHeader() {
    return (
        <Menu borderless secondary id="mainHeader">
            <Menu.Item>
                <Header floated="left" as={Link} to="/">
                    Trow
                </Header>
            </Menu.Item>
        </Menu>
    );
}

export const MemoisedMainHeader = memo(MainHeader);
