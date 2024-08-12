import { ComponentPropsWithoutRef } from "react";

type Props = ComponentPropsWithoutRef<"button"> & {
    children: React.ReactNode;
};

export default function Button({ children, className, ...others }: Props) {
    const cls =
        "text-2xl w-64 p-3 bg-gray-100 mx-auto block rounded text-center disabled:opacity-50 ease-in duration-100 hover:scale-[1.05]";
    return (
        <button className={`${cls} ${className}`} {...others}>
            {children}
        </button>
    );
}
