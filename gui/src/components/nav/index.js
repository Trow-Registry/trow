import React, { useState } from "react";
import { Menu, Icon } from "semantic-ui-react";
import { Link } from "react-router-dom";

export default function NavVertical() {
    const [activeItem, setActiveItem] = useState("repositories");

    const handleItemClick = (e, { name }) => {
        setActiveItem(name);
    };

    return (
        <Menu size="mini" icon="labeled" borderless secondary vertical>
            <Menu.Item
                name="repositories"
                active={activeItem === "repositories"}
                onClick={handleItemClick}
                as={Link}
                to="/repositories"
            >
                <Icon name="cube" />
                Repositories
            </Menu.Item>
            <Menu.Item
                name="about"
                active={activeItem === "about"}
                onClick={handleItemClick}
                as={Link}
                to="/about"
            >
                <Icon name="question circle outline" />
                About
            </Menu.Item>
        </Menu>
    );
}
