from manim import *

from bokeh.palettes import magma

INPUT = [
    935,
    158,
    179,
    729,
    408,
    117,
    611,
    629,
    466,
    746,
    416,
    752,
    685,
    326,
    778,
    402,
    748,
    818,
    677,
    367,
    208,
    906,
    672,
    587,
    577,
    349,
    440,
    331,
    256,
    846,
    354,
    635,
    967,
    865,
    647,
    576,
    207,
    552,
    439,
    219,
]


PIVOT = 500

INITIAL_WAIT = 2.0


def input_iter():
    left = 0

    for right, val in enumerate(INPUT):
        val_is_lt = val < PIVOT

        yield (left, right, val_is_lt)

        if val_is_lt:
            left += 1


class CustomIndicate(Transform):
    def __init__(
        self,
        mobject,
        fill_color: str = None,
        rate_func=there_and_back,
        **kwargs
    ) -> None:
        self.fill_color = fill_color
        super().__init__(mobject, rate_func=rate_func, **kwargs)

    def create_target(self) -> "Mobject":
        target = self.mobject.copy()
        target.fill_color = self.fill_color
        return target


def absdiff(a, b):
    if a >= b:
        return a - b
    else:
        return b - a


def halway_point(a, b):
    half_distance = absdiff(a, b) / 2.0
    return (a + half_distance) if a <= b else (b + half_distance)


def curved_line(start, end, y_offset):
    path = Line(start, end)
    path.points[1] = [
        halway_point(start[0], end[0]),
        start[1] + y_offset,
        0.0,
    ]

    return path


def swap_horizontal(scene, rect_vals, left_offset, right_offset):
    rect_a = rect_vals[left_offset]
    rect_b = rect_vals[right_offset]
    a_pos = rect_a.get_center()
    b_pos = rect_b.get_center()

    a_new_pos = [b_pos[0], a_pos[1], a_pos[2]]
    b_new_pos = [a_pos[0], b_pos[1], b_pos[2]]

    a_path = curved_line(a_pos, a_new_pos, SceneInfo.bar_width(scene) * 2.0)
    b_path = curved_line(b_pos, b_new_pos, SceneInfo.bar_width(scene) * 2.0)

    scene.play(
        MoveAlongPath(rect_a, a_path),
        MoveAlongPath(rect_b, b_path),
        run_time=0.5,
    )

    rect_vals[left_offset], rect_vals[right_offset] = (
        rect_vals[right_offset],
        rect_vals[left_offset],
    )


def replace_horizontal(scene, rect_vals, src_rect_offset, dst_rect_offset):
    src_rect = rect_vals[src_rect_offset]

    src_pos = src_rect.get_center()
    dst_pos = src_rect.get_center()
    dst_pos[0] = rect_vals[dst_rect_offset].get_center()[0]

    path = curved_line(src_pos, dst_pos, SceneInfo.bar_width(scene) * 2.0)

    src_rect_copy = src_rect.copy()

    scene.add(src_rect_copy)
    scene.play(
        FadeOut(rect_vals[dst_rect_offset]),
        MoveAlongPath(src_rect_copy, path),
        run_time=0.5,
    )

    rect_vals[dst_rect_offset] = src_rect_copy


class SceneInfo:
    def padding():
        return 0.15

    def width(scene):
        return scene.camera.frame_width - (SceneInfo.padding() * 2.0)

    def height(scene):
        return scene.camera.frame_height - (SceneInfo.padding() * 2.0)

    def bar_width(scene):
        return SceneInfo.width(scene) / len(INPUT)


def partition_animation(scene, rect_anim_fn):
    palette = magma(100)

    def rect_ctor(input_val, i):
        input_val_range_0_1 = input_val / 1000.0
        fill_color = palette[round(input_val_range_0_1 * 99.0)]
        bar_height = input_val_range_0_1 * SceneInfo.height(scene)

        rect = Rectangle(
            color=BLACK, height=bar_height, width=SceneInfo.bar_width(scene)
        )
        rect.set_fill(fill_color, opacity=1.0)
        rect.move_to([SceneInfo.bar_width(scene) * i, bar_height / 2.0, 0.0])

        return rect

    rect_vals = [rect_ctor(input_val, i) for i, input_val in enumerate(INPUT)]

    scene.camera.frame_center = [
        (SceneInfo.width(scene) / 2.0) - (SceneInfo.bar_width(scene) / 2.0),
        SceneInfo.height(scene) / 2.0,
        0.0,
    ]
    scene.camera.background_color = "#f6f8fa"

    pivot_line = Line(
        color=BLACK,
        start=[
            0.0 - (SceneInfo.bar_width(scene) / 2.0),
            SceneInfo.height(scene) / 2.0,
            0.0,
        ],
        end=[
            SceneInfo.width(scene) - (SceneInfo.bar_width(scene) / 2.0),
            SceneInfo.height(scene) / 2.0,
            0.0,
        ],
    )

    scene.add(pivot_line)

    for rect in rect_vals:
        scene.add(rect)

    rect_anim_fn(scene, rect_vals)

    scene.wait(duration=10.0)


def highlight_color(val_is_lt):
    return "#77b255" if val_is_lt else "#db2e43"


def lomuto_partition_anim(scene, rect_vals):
    scene.wait(duration=INITIAL_WAIT)

    for left_offset, right_offset, val_is_lt in list(input_iter()):
        rect = rect_vals[right_offset]
        color = highlight_color(val_is_lt)

        scene.play(
            CustomIndicate(rect, highlight_color(val_is_lt)),
            run_time=0.5,
        )
        scene.wait(duration=0.25)

        if val_is_lt:
            swap_horizontal(scene, rect_vals, left_offset, right_offset)
            scene.wait(duration=0.25)


class LomutoPartition(Scene):
    def construct(self):
        partition_animation(self, lomuto_partition_anim)


def setup_gap_value(scene, rect_vals):
    screen_extension_height = SceneInfo.bar_width(scene) * 3.0
    scene.camera.frame_height += screen_extension_height
    scene.camera.frame_center[1] = (
        scene.camera.frame_center[1] + screen_extension_height / 2.0
    )

    aux_mem_rect = Rectangle(
        color=BLACK,
        height=SceneInfo.bar_width(scene) * 2.0,
        width=SceneInfo.width(scene),
    )

    aux_mem_rect_pos = [
        scene.camera.frame_center[0],
        SceneInfo.height(scene) - (aux_mem_rect.height / 2.0),
        0.0,
    ]
    aux_mem_rect.move_to(aux_mem_rect_pos)

    scene.add(aux_mem_rect)

    gap_value_txt = Text(
        "gap.value",
        font="Fira Mono",
        font_size=26,
        color=BLACK,
    )
    gap_value_txt.move_to(
        [
            aux_mem_rect_pos[0]
            + (
                ((SceneInfo.width(scene) / 2.0) - SceneInfo.padding())
                - (gap_value_txt.width / 2.0)
            ),
            aux_mem_rect_pos[1],
            0.0,
        ]
    )

    scene.add(gap_value_txt)

    original_gap_val = rect_vals[0].copy()
    gap_value_rect = original_gap_val.copy()
    gap_value_rect.move_to(aux_mem_rect.get_center())
    gap_value_rect.rotate(PI / 2.0)

    return original_gap_val, gap_value_rect


def lomuto_cyc_partition_anim(scene, rect_vals):
    original_gap_val, gap_value_rect = setup_gap_value(scene, rect_vals)

    scene.wait(duration=INITIAL_WAIT)

    for left_offset, right_offset, val_is_lt in list(input_iter()):
        rect = rect_vals[right_offset]
        scene.play(
            CustomIndicate(rect, highlight_color(val_is_lt)),
            run_time=0.5,
        )
        scene.wait(duration=0.25)

        if right_offset == 0:
            scene.play(Create(gap_value_rect))
            continue

        gap_pos = left_offset
        replace_horizontal(scene, rect_vals, right_offset, gap_pos)
        scene.wait(duration=0.25)

        new_gap_pos = gap_pos + (1 if val_is_lt else 0)
        new_left_dst = right_offset if val_is_lt else new_gap_pos

        replace_horizontal(scene, rect_vals, new_gap_pos, new_left_dst)
        scene.wait(duration=0.25)

    gap_rect = rect_vals[new_gap_pos]
    gap_value_end_rect = original_gap_val
    gap_value_end_rect.move_to(
        [gap_rect.get_center()[0], gap_value_end_rect.get_center()[1], 0.0]
    )

    scene.wait(1.0)
    scene.play(
        FadeOut(gap_rect),
        Uncreate(gap_value_rect),
        FadeIn(gap_value_end_rect),
        run_time=2.0,
    )


class LomutoCycPartition(Scene):
    def construct(self):
        partition_animation(self, lomuto_cyc_partition_anim)


def lomuto_cyc_opt_partition_anim(scene, rect_vals):
    original_gap_val, gap_value_rect = setup_gap_value(scene, rect_vals)

    scene.wait(duration=INITIAL_WAIT)

    for left_offset, right_offset, val_is_lt in list(input_iter()):
        rect = rect_vals[right_offset]
        if right_offset == 0:
            gap_value_is_lt = val_is_lt
            scene.play(Create(gap_value_rect))
            continue

        scene.play(
            CustomIndicate(rect, highlight_color(val_is_lt)),
            run_time=0.5,
        )
        scene.wait(duration=0.25)

        gap_pos = right_offset - 1

        replace_horizontal(scene, rect_vals, left_offset, gap_pos)
        scene.wait(duration=0.25)

        replace_horizontal(scene, rect_vals, right_offset, left_offset)
        scene.wait(duration=0.25)

        next_left = left_offset + (1 if val_is_lt else 0)

    scene.wait(1.0)
    replace_horizontal(scene, rect_vals, next_left, right_offset)

    gap_rect = rect_vals[next_left]
    gap_value_end_rect = original_gap_val
    gap_value_end_rect.move_to(
        [gap_rect.get_center()[0], gap_value_end_rect.get_center()[1], 0.0]
    )

    scene.wait(1.0)
    scene.play(
        FadeOut(gap_rect),
        Uncreate(gap_value_rect),
        FadeIn(gap_value_end_rect),
        run_time=2.0,
    )

    scene.play(
        CustomIndicate(gap_value_end_rect, highlight_color(gap_value_is_lt)),
        run_time=0.5,
    )


class LomutoCycOptPartition(Scene):
    def construct(self):
        partition_animation(self, lomuto_cyc_opt_partition_anim)


def hoare_partition_anim(scene, rect_vals):
    comp_results = [val_is_lt for _, _, val_is_lt in list(input_iter())]

    left = 0
    right = len(INPUT)

    scene.wait(duration=INITIAL_WAIT)

    while True:
        while left < right:
            val_is_lt = comp_results[left]
            scene.play(
                CustomIndicate(rect_vals[left], highlight_color(val_is_lt)),
                run_time=0.5,
            )
            scene.wait(duration=0.25)

            if not val_is_lt:
                break

            left += 1

        scene.wait(duration=0.35)

        while True:
            right -= 1

            if left >= right:
                break

            val_is_lt = comp_results[right]
            scene.play(
                CustomIndicate(rect_vals[right], highlight_color(val_is_lt)),
                run_time=0.5,
            )
            scene.wait(duration=0.25)

            if val_is_lt:
                break

        if left >= right:
            break

        scene.wait(duration=0.35)
        swap_horizontal(scene, rect_vals, left, right)

        left += 1


class HoarePartition(Scene):
    def construct(self):
        partition_animation(self, hoare_partition_anim)
