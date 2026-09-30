from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path

from PIL import Image


def normalize_color(
    pixel: tuple[int, int, int, int],
) -> str | None:
    """
    Convert an RGBA pixel into a board color.

    Fully transparent pixels become cell ID 0.

    All other pixels are represented as #RRGGBB.
    """
    red, green, blue, alpha = pixel

    if alpha == 0:
        return None

    return f"#{red:02x}{green:02x}{blue:02x}"


def bitmap_to_board(image_path: Path) -> dict:
    """
    Convert a bitmap image into the board JSON structure.

    Board format:

        {
            "width": 32,
            "height": 32,
            "colors": [
                "#ff0000",
                "#00ff00"
            ],
            "data": [
                0,
                1,
                2
            ]
        }

    Cell ID 0 is reserved for transparent / empty cells.

    colors[0] corresponds to cell ID 1,
    colors[1] corresponds to cell ID 2,
    and so on.
    """
    if not image_path.exists():
        raise FileNotFoundError(
            f"Image does not exist: {image_path}"
        )

    if not image_path.is_file():
        raise ValueError(
            f"Image path is not a file: {image_path}"
        )

    with Image.open(image_path) as source:
        image = source.convert("RGBA")

        width, height = image.size

        colors: list[str] = []
        color_ids: dict[str, int] = {}

        data: list[int] = []

        for pixel in image.getdata():
            color = normalize_color(pixel)

            if color is None:
                data.append(0)
                continue

            color_id = color_ids.get(color)

            if color_id is None:
                color_id = len(colors) + 1

                colors.append(color)
                color_ids[color] = color_id

            data.append(color_id)

    return {
        "width": width,
        "height": height,
        "colors": colors,
        "data": data,
    }


def send_board(
    board: dict,
    route: str,
    timeout: float,
) -> tuple[int, str]:
    """
    POST a board buffer to the specified route.
    """
    body = json.dumps(
        board,
        separators=(",", ":"),
    ).encode("utf-8")

    request = urllib.request.Request(
        route,
        data=body,
        method="POST",
        headers={
            "Content-Type": "application/json",
            "Accept": "application/json",
            "User-Agent": "SMN-Board-Tool/1.0",
        },
    )

    try:
        with urllib.request.urlopen(
            request,
            timeout=timeout,
        ) as response:
            response_body = response.read().decode(
                "utf-8",
                errors="replace",
            )

            return response.status, response_body

    except urllib.error.HTTPError as error:
        response_body = error.read().decode(
            "utf-8",
            errors="replace",
        )

        raise RuntimeError(
            f"Server returned HTTP {error.code}: "
            f"{response_body}"
        ) from error

    except urllib.error.URLError as error:
        raise RuntimeError(
            f"Could not connect to {route}: "
            f"{error.reason}"
        ) from error


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Convert a bitmap image into an SMN board buffer "
            "and send it to a board/set route."
        )
    )

    parser.add_argument(
        "image",
        type=Path,
        help="Path to the bitmap image.",
    )

    parser.add_argument(
        "route",
        help=(
            "Full board/set URL, for example "
            "http://127.0.0.1:8000/board/set"
        ),
    )

    parser.add_argument(
        "--timeout",
        type=float,
        default=10.0,
        help="HTTP timeout in seconds. Default: 10.",
    )

    return parser.parse_args()


def main() -> None:
    args = parse_arguments()

    try:
        board = bitmap_to_board(
            args.image,
        )

        print(
            f"Converted {args.image}: "
            f"{board['width']}x{board['height']}, "
            f"{len(board['colors'])} colors"
        )

        status, response_body = send_board(
            board=board,
            route=args.route,
            timeout=args.timeout,
        )

        print(
            f"Board sent successfully: HTTP {status}"
        )

        if response_body:
            print(response_body)

    except Exception as error:
        print(
            f"Error: {error}",
            file=sys.stderr,
        )

        sys.exit(1)


if __name__ == "__main__":
    main()
