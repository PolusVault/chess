import Modal from "react-modal";

const customStyles = {
    overlay: {
        backgroundColor: "rgba(0, 0, 0, 0.1)",
    },
    content: {
        top: "50%",
        left: "50%",
        right: "auto",
        bottom: "auto",
        transform: "translate(-50%, -50%)",
    },
};

Modal.setAppElement("#root");

function MyModal({ children, ...props }: Modal.Props) {
    return (
        <Modal
            {...props}
            shouldCloseOnOverlayClick
            shouldCloseOnEsc
            style={props.style ? props.style : customStyles}
        >
            <div className="text-lg">{children}</div>
        </Modal>
    );
}

export default MyModal;
